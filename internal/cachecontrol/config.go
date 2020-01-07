// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package cachecontrol

import (
	"fmt"
	"net/http"
	"strings"
	"time"
)

var (
	sep = ", "
)

// Config represent a cache configuration
// that will generate the appropriate HTTP headers
type Config struct {
	NoStore        bool
	NoCache        bool
	Public         bool
	MaxAge         time.Duration
	MustRevalidate bool
	NoTransform    bool
	LastModified   time.Time
	Immutable      bool
	ETag           string
}

// Write applies the given Config to a http.ResponseWriter
func (c *Config) Write(r http.ResponseWriter) {
	if c.ETag != "" {
		r.Header().Set(HeaderETag, c.ETag)
	}

	var cc strings.Builder
	if c.Public {
		cc.WriteString("public")
	} else {
		cc.WriteString("private")
	}
	if c.NoStore {
		cc.WriteString(sep)
		cc.WriteString("no-store")
		cc.WriteString(sep)
		cc.WriteString("max-age=0")
		r.Header().Set(HeaderCacheControl, cc.String())
		r.Header().Set(HeaderPragma, "no-cache")
		return
	}

	if c.NoCache {
		cc.WriteString(sep)
		cc.WriteString("no-cache")
		cc.WriteString(sep)
		cc.WriteString("max-age=0")
		r.Header().Set(HeaderCacheControl, cc.String())
		r.Header().Set(HeaderPragma, "no-cache")
		return
	}

	if c.Immutable {
		// 0.9 years
		d := 7884 * time.Hour
		exp := time.Now().UTC().Add(d).Format(time.RFC1123)
		cc.WriteString(sep)
		cc.WriteString("immutable")
		cc.WriteString(sep)
		cc.WriteString(fmt.Sprintf("max-age=%.0f", d.Seconds()))
		r.Header().Set(HeaderCacheControl, cc.String())
		r.Header().Set(HeaderExpires, exp)
		return
	}

	if c.MaxAge != 0 {
		a := fmt.Sprintf("%.0f", c.MaxAge.Seconds())
		cc.WriteString(sep)
		cc.WriteString(fmt.Sprintf("max-age=%s", a))
		cc.WriteString(sep)
		cc.WriteString(fmt.Sprintf("s-maxage=%s", a))
		r.Header().Set(HeaderExpires, time.Now().UTC().Add(c.MaxAge).Format(time.RFC1123))
	}

	if c.MustRevalidate {
		cc.WriteString(sep)
		cc.WriteString("must-revalidate")
	}

	if c.NoTransform {
		cc.WriteString(sep)
		cc.WriteString("no-transform")
	}

	if !c.LastModified.IsZero() {
		r.Header().Set(HeaderLastModified, c.LastModified.UTC().Format(time.RFC1123))
	}
	r.Header().Set(HeaderCacheControl, cc.String())
}

func NoCache(public bool) *Config {
	return &Config{
		Public:  public,
		NoCache: true,
	}
}

func NoStore(public bool) *Config {
	return &Config{
		Public:  public,
		NoStore: true,
	}
}
