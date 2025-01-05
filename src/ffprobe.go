package main

import (
	"encoding/json"
	"os/exec"
)

type ProbeResult struct {
	Streams []ProbeStream `json:"streams"`
	Format  ProbeFormat   `json:"format"`
}

type ProbeStream struct {
	Index     int    `json:"index"`
	CodecName string `json:"codec_name"`
	CodecType string `json:"codec_type"`
	Channels  *int   `json:"channels"`
}

type ProbeFormat struct {
	FormatName     string `json:"format_name"`
	FormatLongName string `json:"format_long_name"`
	Duration       string `json:"duration"`
}

func ffprobe(task *Task) error {
	cmd := exec.Command("ffprobe", "-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", task.Path)
	stdout, err := cmd.Output()
	if err != nil {
		return err
	}
	var probe ProbeResult
	err = json.Unmarshal(stdout, &probe)
	if err != nil {
		return err
	}
	task.Probe = &probe
	return nil
}
