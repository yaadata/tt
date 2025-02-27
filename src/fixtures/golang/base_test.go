package golang

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

// comment
func TestBaseCase(t *testing.T) {
	// arrange
	a := 2
	b := 3
	// act
	res := sample_add(a, b)
	// assert
	assert.Equal(t, 5, res)
}
