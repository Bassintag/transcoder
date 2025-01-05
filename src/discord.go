package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"time"
)

type DiscordPayload struct {
	Embeds []DiscordEmbed `json:"embeds"`
}

type DiscordEmbed struct {
	Title  string         `json:"title"`
	Fields []DiscordField `json:"fields"`
	Color  int            `json:"color"`
}

type DiscordField struct {
	Name   string `json:"name"`
	Value  string `json:"value"`
	Inline bool   `json:"inline"`
}

type DiscordResponse struct {
	Id string `json:"id"`
}

func formatDuration(seconds float64) string {
	duration := time.Duration(int(seconds * 1e9))
	return fmt.Sprintf("%s", duration)
}

const progressBarLength = 20

func formatProgressBar(ratio float64) string {
	var buffer bytes.Buffer
	for i := 0; i < progressBarLength; i++ {
		p := float64(i) / progressBarLength
		if p >= ratio {
			buffer.WriteRune('░')
		} else {
			buffer.WriteRune('█')
		}
	}
	buffer.WriteString(
		fmt.Sprintf(" %.2f%%", ratio*100),
	)
	return buffer.String()
}

func makePayloadBuffer(task *Task) *bytes.Buffer {
	fields := []DiscordField{
		{Name: "Path", Value: task.Path, Inline: false},
		{Name: "Output path", Value: task.OutputPath, Inline: false},
	}
	if task.Status == PROCESSING && task.Probe != nil {
		seconds, _ := strconv.ParseFloat(task.Probe.Format.Duration, 64)
		fields = append(fields, DiscordField{
			Name: "Duration", Value: formatDuration(seconds), Inline: true,
		})
		if task.Progress != nil {
			fields = append(fields, DiscordField{
				Name: "Timestamp", Value: formatDuration(task.Progress.Timestamp), Inline: true,
			}, DiscordField{
				Name: "Speed", Value: task.Progress.Speed, Inline: true,
			}, DiscordField{
				Name: "Progress", Value: formatProgressBar(
					task.Progress.Timestamp / seconds,
				), Inline: false,
			})
		}
	}
	var color int
	switch task.Status {
	case QUEUED:
		color = 0xa855f7
	case PROCESSING:
		color = 0xf97316
	case DONE:
		color = 0x22c55e
	case FAILED:
		color = 0xef4444
	default:
		color = 0x737373
	}
	payload := DiscordPayload{
		Embeds: []DiscordEmbed{
			{
				Title:  "Transcoding file",
				Fields: fields,
				Color:  color,
			},
		},
	}
	jsonData, _ := json.Marshal(payload)
	return bytes.NewBuffer(jsonData)
}

type DiscordClient struct {
	webhookUrl string
	httpClient *http.Client
}

func NewDiscordClient(webhookUrl string) *DiscordClient {
	return &DiscordClient{webhookUrl: webhookUrl, httpClient: &http.Client{}}
}

func (client *DiscordClient) Trigger(task *Task) {
	payload := makePayloadBuffer(task)
	method := "POST"
	url := client.webhookUrl
	if task.WebhookId != nil {
		method = "PATCH"
		url = client.webhookUrl + "/messages/" + *task.WebhookId
	}
	req, _ := http.NewRequest(method, url, payload)
	req.Header.Set("Content-Type", "application/json")
	req.URL.RawQuery = "wait=true"
	resp, _ := client.httpClient.Do(req)
	defer resp.Body.Close()
	data := DiscordResponse{}
	_ = json.NewDecoder(resp.Body).Decode(&data)
	task.WebhookId = &data.Id
}
