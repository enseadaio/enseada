package get

import (
	"github.com/spf13/cobra"
)

var RootCmd = &cobra.Command{
	Use:   "get [resource]",
	Short: "Get a resource",
}

func init() {
	RootCmd.AddCommand(getMvnRepo)
}
