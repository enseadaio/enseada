package guid

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestNew(t *testing.T) {
	guid := New("test", "test")
	assert.Equal(t, "test", guid.db)
	assert.Equal(t, "test", guid.id)
}

func TestNewWithRev(t *testing.T) {
	guid := NewWithRev("test", "test", "1")
	assert.Equal(t, "test", guid.db)
	assert.Equal(t, "test", guid.id)
	assert.Equal(t, "1", guid.rev)
}

func TestParseWithRev(t *testing.T) {
	guid, err := Parse("test://test?rev=1")
	assert.NoError(t, err)
	assert.Equal(t, "test", guid.db)
	assert.Equal(t, "test", guid.id)
	assert.Equal(t, "1", guid.rev)
}

func TestParse(t *testing.T) {
	guid, err := Parse("test://test")
	assert.NoError(t, err)
	assert.Equal(t, "test", guid.db)
	assert.Equal(t, "test", guid.id)
}

func TestParseInvalid(t *testing.T) {
	_, err := Parse("test")
	assert.Error(t, err)
	assert.Equal(t, "is missing database", err.Error())

	_, err = Parse("test://")
	assert.Error(t, err)
	assert.Equal(t, "is missing ID", err.Error())
}

func TestParseEmpty(t *testing.T) {
	_, err := Parse("")
	assert.Error(t, err)
	assert.Equal(t, "GUID can't be blank", err.Error())
}

func TestGUID_String(t *testing.T) {
	guid := New("test", "test")
	assert.Equal(t, "test://test", guid.String())
}

func TestGUID_StringWithRev(t *testing.T) {
	guid := NewWithRev("test", "test", "1")
	assert.Equal(t, "test://test?rev=1", guid.String())
}
