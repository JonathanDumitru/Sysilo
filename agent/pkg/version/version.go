package version

// Build information set via ldflags at compile time
var (
	Version   = "dev"
	Commit    = "unknown"
	BuildDate = "unknown"
)

// Info returns the full version information as a string
func Info() string {
	return Version + " (commit: " + Commit + ", built: " + BuildDate + ")"
}
