package main

import (
	"log"
	"os"
	"sync"
)

type TaskStatus int

const (
	QUEUED = iota
	PROCESSING
	DONE
	FAILED
)

type Task struct {
	Path       string
	OutputPath string
	Probe      *ProbeResult
	Progress   *TranscodingProgress
	WebhookId  *string
	Status     TaskStatus
}

func TaskNew(path string, outputPath string) Task {
	return Task{
		Path:       path,
		OutputPath: outputPath,
		Status:     QUEUED,
	}
}

func (task *Task) run(mu *sync.Mutex) error {
	discord := NewDiscordClient(os.Getenv("DISCORD_WEBHOOK"))
	discord.Trigger(task)
	mu.Lock()
	defer mu.Unlock()
	task.Status = PROCESSING
	err := ffprobe(task)
	if err != nil {
		log.Println("ERR: ffprobe failed for file: " + task.Path)
		task.Status = FAILED
		discord.Trigger(task)
		return err
	}
	discord.Trigger(task)
	err = ffmpeg(task, func() {
		discord.Trigger(task)
	})
	if err != nil {
		log.Println("ERR: ffmpeg failed for file: " + task.Path)
		task.Status = FAILED
		discord.Trigger(task)
		return err
	}
	_ = os.Remove(task.Path)
	task.Status = DONE
	discord.Trigger(task)
	return nil
}
