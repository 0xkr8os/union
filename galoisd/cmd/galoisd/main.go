package main

import (
	"galois/cmd/galoisd/cmd"
	"github.com/spf13/cobra"
)

func main() {
	var rootCmd = &cobra.Command{Use: "galoisd"}
	rootCmd.AddCommand(cmd.ServeCmd())
	rootCmd.AddCommand(cmd.GenContract())
	rootCmd.AddCommand(cmd.ExampleProveCmd())
	rootCmd.AddCommand(cmd.ExampleProve16Cmd())
	rootCmd.AddCommand(cmd.ExampleVerifyCmd())
	rootCmd.AddCommand(cmd.QueryStats())
	rootCmd.Execute()
}
