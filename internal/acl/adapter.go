package acl

import (
	"context"
	"github.com/casbin/casbin/v2/model"
	"github.com/casbin/casbin/v2/persist"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type CasbinRule struct {
	Id    string `json:"_id,omitempty"`
	Rev   string `json:"_rev,omitempty"`
	PType string `json:"PType,omitempty"`
	V0    string `json:"V0,omitempty"`
	V1    string `json:"V1,omitempty"`
	V2    string `json:"V2,omitempty"`
	V3    string `json:"V3,omitempty"`
	V4    string `json:"V4,omitempty"`
	V5    string `json:"V5,omitempty"`
}

type Adapter struct {
	logger echo.Logger
	client *kivik.Client
	dbname string
	policy []CasbinRule
}

func NewAdapter(client *kivik.Client, dbname string, logger echo.Logger) (*Adapter, error) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	exist, err := client.DBExists(ctx, dbname)
	if err != nil {
		return nil, err
	}
	if !exist {
		err = client.CreateDB(ctx, dbname)
		if err != nil {
			return nil, err
		}
	}

	return &Adapter{
		client: client,
		dbname: dbname,
		logger: logger,
	}, nil
}

func (a *Adapter) LoadPolicy(model model.Model) error {
	err := a.loadFromDatabase()
	if err != nil {
		return err
	}

	for _, line := range a.policy {
		loadPolicyLine(line, model)
	}
	return nil
}

func (a *Adapter) SavePolicy(model model.Model) error {
	err := a.loadFromDatabase()
	if err != nil {
		return err
	}

	ctx := context.Background()
	err = a.client.DestroyDB(ctx, a.dbname)
	if err != nil {
		return err
	}

	err = a.client.CreateDB(ctx, a.dbname)
	if err != nil {
		return err
	}

	a.policy = []CasbinRule{}
	var lines []CasbinRule
	for pType, ast := range model["p"] {
		for _, rule := range ast.Policy {
			line := savePolicyLine(pType, rule)
			lines = append(lines, line)
		}
	}

	for pType, ast := range model["g"] {
		for _, rule := range ast.Policy {
			line := savePolicyLine(pType, rule)
			lines = append(lines, line)
		}
	}

	a.policy = lines
	err = a.saveToDatabase()
	return err
}

func (a *Adapter) AddPolicy(sec string, ptype string, rule []string) error {
	a.logger.Infof("adding new policy sec: %s, ptype: %s, rule: %v", sec, ptype, rule)
	ctx := context.Background()
	line := savePolicyLine(ptype, rule)
	a.logger.Debugf("%+v", line)
	db := a.client.DB(ctx, a.dbname)
	_, err := db.Put(ctx, line.Id, line)
	if err != nil {
		return err
	}

	return a.loadFromDatabase()
}

func (a *Adapter) RemovePolicy(sec string, ptype string, rule []string) error {
	line := savePolicyLine(ptype, rule)
	err := a.deleteLineFromDatabase(line)
	if err != nil {
		return err
	}
	return a.loadFromDatabase()
}

func (a *Adapter) RemoveFilteredPolicy(sec string, ptype string, fieldIndex int, fieldValues ...string) error {
	line := CasbinRule{PType: ptype}

	idx := fieldIndex + len(fieldValues)
	if fieldIndex <= 0 && idx > 0 {
		line.V0 = fieldValues[0-fieldIndex]
	}
	if fieldIndex <= 1 && idx > 1 {
		line.V1 = fieldValues[1-fieldIndex]
	}
	if fieldIndex <= 2 && idx > 2 {
		line.V2 = fieldValues[2-fieldIndex]
	}
	if fieldIndex <= 3 && idx > 3 {
		line.V3 = fieldValues[3-fieldIndex]
	}
	if fieldIndex <= 4 && idx > 4 {
		line.V4 = fieldValues[4-fieldIndex]
	}
	if fieldIndex <= 5 && idx > 5 {
		line.V5 = fieldValues[5-fieldIndex]
	}

	err := a.deleteLineFromDatabase(line)
	if err != nil {
		return err
	}

	return a.loadFromDatabase()
}

func (a *Adapter) loadFromDatabase() error {
	a.logger.Debug("loading rules from database")
	var policy []CasbinRule
	ctx := context.Background()
	db := a.client.DB(ctx, a.dbname)
	rows, err := db.AllDocs(ctx, kivik.Options{
		"include_docs": true,
	})
	if err != nil {
		return err
	}

	a.logger.Debug("fetched rules from database. Attempting to load")
	for rows.Next() {
		var line CasbinRule
		if err := rows.ScanDoc(&line); err != nil {
			return err
		}
		a.logger.Debugf("loaded rule %+v", line)
		policy = append(policy, line)
	}

	a.policy = policy
	return nil
}

func (a *Adapter) saveToDatabase() error {
	ctx := context.Background()
	db := a.client.DB(ctx, a.dbname)
	for i, line := range a.policy {
		if line.Id == "" {
			line.Id = lineToText(line)
		}

		rev, err := db.Put(ctx, line.Id, line)
		if err != nil {
			return err
		}
		a.policy[i].Rev = rev
	}
	return nil
}

func loadPolicyLine(line CasbinRule, model model.Model) {
	lineText := lineToText(line)
	persist.LoadPolicyLine(lineText, model)
}

func savePolicyLine(ptype string, rule []string) CasbinRule {
	line := CasbinRule{}

	line.PType = ptype
	if len(rule) > 0 {
		line.V0 = rule[0]
	}
	if len(rule) > 1 {
		line.V1 = rule[1]
	}
	if len(rule) > 2 {
		line.V2 = rule[2]
	}
	if len(rule) > 3 {
		line.V3 = rule[3]
	}
	if len(rule) > 4 {
		line.V4 = rule[4]
	}
	if len(rule) > 5 {
		line.V5 = rule[5]
	}

	line.Id = lineToText(line)

	return line
}

func lineToText(line CasbinRule) string {
	lineText := line.PType
	if line.V0 != "" {
		lineText += ", " + line.V0
	}
	if line.V1 != "" {
		lineText += ", " + line.V1
	}
	if line.V2 != "" {
		lineText += ", " + line.V2
	}
	if line.V3 != "" {
		lineText += ", " + line.V3
	}
	if line.V4 != "" {
		lineText += ", " + line.V4
	}
	if line.V5 != "" {
		lineText += ", " + line.V5
	}
	return lineText
}

func (a *Adapter) deleteLineFromDatabase(line CasbinRule) error {
	ctx := context.Background()
	if line.Id == "" {
		line.Id = lineToText(line)
	}

	db := a.client.DB(ctx, a.dbname)
	_, rev, err := db.GetMeta(ctx, line.Id)
	if err != nil {
		return err
	}

	_, err = db.Delete(ctx, line.Id, rev)
	return err
}
