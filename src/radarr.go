package main

type RadarrMovie struct {
	Title      string `json:"title" binding:"required"`
	FolderPath string `json:"folderPath" binding:"required"`
}

type RadarrMovieFile struct {
	RelativePath string `json:"relativePath" binding:"required"`
}

type RadarrEvent struct {
	Movie     *RadarrMovie     `json:"movie" binding:"required"`
	MovieFile *RadarrMovieFile `json:"movieFile" binding:"required"`
}
