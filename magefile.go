// +build mage

package main

import (
	"fmt"
	"github.com/magefile/mage/mg" // mg contains helpful utility functions, like Deps
	"os"
	"os/exec"
)

// Default target to run when none is specified
// If not set, running mage will list available targets
var Default = Build

// A build step that requires additional params, or platform specific steps for example
func Build() error {
	mg.Deps(BuildClient)
	mg.Deps(BuildServer)
	return nil
}

func BuildServer() error {
	mg.Deps(InstallDeps)
	fmt.Println("Building server executable...")
	os.MkdirAll("./bin", 0700)
	cmd := exec.Command("go", "build", "-o", "bin/enseada-server", "./cmd/enseada-server")
	return cmd.Run()
}

func BuildClient() error {
	mg.Deps(InstallDeps)
	fmt.Println("Building client executable...")
	os.MkdirAll("./bin", 0700)
	cmd := exec.Command("go", "build", "-o", "bin/enseada", "./cmd/enseada")
	return cmd.Run()
}

// A custom install step if you need your bin someplace other than go/bin
func Install() error {
	mg.Deps(Build)
	fmt.Println("Installing...")
	return os.Rename("./bin/enseada-server", "/usr/bin/enseada-server")
}

// Manage your deps, or running package managers.
func InstallDeps() error {
	fmt.Println("Installing Deps...")
	cmd := exec.Command("go", "mod", "vendor")
	return cmd.Run()
}

// Clean up after yourself
func Clean() {
	fmt.Println("Cleaning...")
	os.RemoveAll("bin")
}
