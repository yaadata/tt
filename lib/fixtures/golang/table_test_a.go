package golang

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func sample_add(a, b int) int {
	return a + b
}

func TestTableTest(t *testing.T) {
	for _, tt := range []struct {
		description string
		a           int
		b           int
		expected    int
	}{
		{
			description: "base case",
			a:           0,
			b:           3,
			expected:    3,
		},
		{
			description: "case 1",
			a:           1,
			b:           3,
			expected:    4,
		},
	} {
		t.Run(tt.description, func(t *testing.T) {
			actual := sample_add(tt.a, tt.b)
			assert.Equal(t, tt.expected, actual)
		})
	}
}
