package main

import (
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"os"
	"path"
	"strings"
	"sync"
)

var mu = sync.Mutex{}

func postTask(c *gin.Context) {
	c.Status(http.StatusOK)
	var event RadarrEvent
	if err := c.ShouldBindJSON(&event); err != nil {
		return
	}
	rootFolderPath := os.Getenv("ROOT_FOLDER")
	folderPath := path.Join(
		rootFolderPath,
		event.Movie.FolderPath,
	)
	taskPath := path.Join(
		folderPath,
		event.MovieFile.RelativePath,
	)
	outputRelativePath := strings.ReplaceAll(event.Movie.Title, " ", ".") + ".h264.aac.stereo.remux.mp4"
	outputPath := path.Join(
		folderPath,
		outputRelativePath,
	)
	task := TaskNew(taskPath, outputPath)
	go task.run(&mu)
}

func api() {
	router := gin.Default()
	router.POST("/tasks", postTask)

	err := router.Run()
	if err != nil {
		log.Fatal(err)
	}
}
