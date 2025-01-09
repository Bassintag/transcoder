package main

import (
	"bufio"
	"log"
	"os/exec"
	"strconv"
	"strings"
)

type TranscodingProgress struct {
	Speed     string
	Timestamp float64
}

const ffmpegArgs = "" +
	"-movflags faststart -hide_banner -loglevel error -progress - -nostats -stats_period 5 " +
	"-c:v libx264 -crf 23 -profile:v baseline -level 3.0 -pix_fmt yuv420p " +
	"-c:a aac -ac 2 -b:a 128k " +
	"-c:s mov_text"

func ffmpeg(task Task, onUpdate func()) error {
	args := []string{"-i", task.Path}
	args = append(args, strings.Split(ffmpegArgs, " ")...)
	args = append(args, task.OutputPath)

	task.Progress = &TranscodingProgress{
		Speed:     "1x",
		Timestamp: 0,
	}

	cmd := exec.Command("ffmpeg", args...)
	log.Println(cmd.String())
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return err
	}
	err = cmd.Start()
	if err != nil {
		return err
	}
	scanner := bufio.NewScanner(stdout)
	scanner.Split(bufio.ScanLines)
	for scanner.Scan() {
		line := scanner.Text()
		parts := strings.Split(line, "=")
		if len(parts) != 2 {
			continue
		}
		key := parts[0]
		value := parts[1]
		switch key {
		case "speed":
			task.Progress.Speed = value
		case "out_time_ms":
			ms, _ := strconv.Atoi(value)
			task.Progress.Timestamp = float64(ms) / 1e6
		case "progress":
			if value == "continue" {
				go onUpdate()
			}
		}
	}
	err = cmd.Wait()
	if err != nil {
		return err
	}
	return nil
}
