// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package edit

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	jww "github.com/spf13/jwalterweatherman"
	"gopkg.in/yaml.v2"
)

// Adapted from this useful article
// https://samrapdev.com/capturing-sensitive-input-with-editor-in-golang-from-the-cli/

const DefaultEditor = "vi"

var header = []byte(`# Please edit the object below. Lines beginning with a '#' will be ignored,
# and an empty file will abort the edit. If an error occurs while saving this file will be
# reopened with the relevant failures.
#
`)

// OpenInEditor takes a pointer to a struct that is JSON-serializable
// using encoding/json and opens it in a text editor as a YAML file.
// The struct will be updated with any change the user has made.
// It returns false if no changes have been made.
func OpenInEditor(obj interface{}) (bool, error) {
	edn := os.Getenv("EDITOR")
	if edn == "" {
		jww.WARN.Println("EDITOR not set. Defaulting to", DefaultEditor)
	}

	ed, err := exec.LookPath(edn)
	if err != nil {
		return false, fmt.Errorf("could not find editor. %s", err.Error())
	}

	file, err := ioutil.TempFile(os.TempDir(), "*.yml")
	if err != nil {
		return false, err
	}
	filename := file.Name()
	defer os.Remove(filename)

	if _, err := file.Write(header); err != nil {
		file.Close()
		return false, err
	}

	in, err := marshal(obj)
	if err != nil {
		file.Close()
		return false, err
	}

	if _, err := file.Write(in); err != nil {
		file.Close()
		return false, err
	}

	if err = file.Close(); err != nil {
		return false, err
	}

	if err := openFileInEditor(ed, filename); err != nil {
		return false, fmt.Errorf("could not open editor. %s", err.Error())
	}

	out, err := ioutil.ReadFile(filename)
	if err != nil {
		return false, err
	}

	out = bytes.TrimPrefix(out, header)
	if bytes.Equal(in, out) {
		return false, nil
	}

	return true, unmarshal(out, obj)
}

func openFileInEditor(ed string, filename string) error {
	cmd := exec.Command(ed, resolveEditorArguments(ed, filename)...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func resolveEditorArguments(ed string, filename string) []string {
	exe := filepath.Base(ed)
	switch {
	case exe == "code", strings.Contains(exe, "Visual Studio Code"):
		return []string{"--wait", filename}
	default:
		return []string{filename}
	}
}

// We do this gymkhana because ProtoBuf structs don't have tags for YAML.
// Marshalling directly to/from YAML would result in internal information being
// written to the file. By using JSON as an intermediate format we make sure that
// the struct fields are properly handled.

func marshal(v interface{}) ([]byte, error) {
	out, err := json.Marshal(v)
	if err != nil {
		return []byte{}, err
	}

	m := make(map[string]interface{})
	if err := json.Unmarshal(out, &m); err != nil {
		return []byte{}, err
	}

	return yaml.Marshal(&m)
}

func unmarshal(in []byte, v interface{}) error {
	m := make(map[string]interface{})
	if err := yaml.Unmarshal(in, &m); err != nil {
		return err
	}

	jin, err := json.Marshal(m)
	if err != nil {
		return err
	}
	return json.Unmarshal(jin, v)
}
