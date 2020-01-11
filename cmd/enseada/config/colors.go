// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package config

import (
	"github.com/jedib0t/go-pretty/table"
	"github.com/jedib0t/go-pretty/text"
)

var TableColorStyle = table.Style{
	Name: "StyleColoredSuperDark",
	Box:  table.StyleBoxDefault,
	Color: table.ColorOptions{
		IndexColumn:  text.Colors{text.FgHiCyan},
		Footer:       text.Colors{text.FgCyan},
		Header:       text.Colors{text.FgHiCyan},
		Row:          text.Colors{text.FgHiWhite},
		RowAlternate: text.Colors{text.FgWhite},
	},
	Format:  table.FormatOptionsDefault,
	Options: table.OptionsNoBordersAndSeparators,
	Title:   table.TitleOptionsDark,
}
