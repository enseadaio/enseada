// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import "github.com/casbin/casbin/v2"

// CasbinTransact suspends auto saving and executes the given function
// before saving the policy and re-enabling auto saving.
func CasbinTransact(e *casbin.Enforcer, f func(e *casbin.Enforcer) error) error {
	// Avoid having race conditions between each action and the watcher.
	// Basically, first apply all changes to the policy, then save the policy.
	e.EnableAutoSave(false)
	if err := f(e); err != nil {
		return err
	}

	if err := e.SavePolicy(); err != nil {
		return err
	}
	e.EnableAutoSave(true)
	return nil
}
