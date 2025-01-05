package main

import (
	"github.com/fsnotify/fsnotify"
	"log"
	"os"
	"path/filepath"
	"strings"
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

func handleFile(path string, discord *DiscordClient, mu *sync.Mutex) {
	if strings.HasSuffix(path, ".out.mp4") {
		return
	}
	ext := filepath.Ext(path)
	outputPath := path[:len(path)-len(ext)] + ".out.mp4"
	task := &Task{Path: path, OutputPath: outputPath, Status: QUEUED}
	discord.Trigger(task)
	mu.Lock()
	defer mu.Unlock()
	task.Status = PROCESSING
	err := ffprobe(task)
	if err != nil {
		log.Println("ERR: ffprobe failed for file: " + path)
		task.Status = FAILED
		discord.Trigger(task)
		return
	}
	discord.Trigger(task)
	err = ffmpeg(task, func() {
		discord.Trigger(task)
	})
	if err != nil {
		log.Println("ERR: ffmpeg failed for file: " + path)
		task.Status = FAILED
		discord.Trigger(task)
		return
	}
	_ = os.Remove(task.Path)
	task.Status = DONE
	discord.Trigger(task)
}

func main() {
	discord := NewDiscordClient(os.Getenv("DISCORD_WEBHOOK"))

	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		log.Fatal(err)
	}
	defer watcher.Close()

	mu := sync.Mutex{}

	go func() {
		for {
			select {
			case event, ok := <-watcher.Events:
				if !ok {
					return
				}
				if event.Has(fsnotify.Create) {
					log.Println("Created file:", event.Name)
					go handleFile(event.Name, discord, &mu)
				}
			case err, ok := <-watcher.Errors:
				if !ok {
					return
				}
				log.Println("error:", err)
			}
		}
	}()

	watchPath, _ := filepath.Abs(os.Getenv("ROOT_FOLDER"))
	log.Println("Watching folder: " + watchPath)
	err = watcher.Add(watchPath)
	if err != nil {
		log.Fatal(err)
	}

	<-make(chan struct{})
}
