package domain

import "container/heap"

// Pair represents a pair of integers
type Pair struct {
	First  int
	Second int
}

// PairHeap is a min-heap of Pairs based on the First value
type PairHeap []Pair

func (h PairHeap) Len() int           { return len(h) }
func (h PairHeap) Less(i, j int) bool { return h[i].First < h[j].First }
func (h PairHeap) Swap(i, j int)      { h[i], h[j] = h[j], h[i] }

func (h *PairHeap) Push(x interface{}) {
	*h = append(*h, x.(Pair))
}

func (h *PairHeap) Pop() interface{} {
	old := *h
	n := len(old)
	x := old[n-1]
	*h = old[0 : n-1]
	return x
}

// MinHeap wraps PairHeap with convenient methods
type MinHeap struct {
	h *PairHeap
}

// NewMinHeap creates a new MinHeap
func NewMinHeap() *MinHeap {
	h := &PairHeap{}
	heap.Init(h)
	return &MinHeap{h: h}
}

// Push adds a pair to the heap
func (m *MinHeap) Push(first, second int) {
	heap.Push(m.h, Pair{First: first, Second: second})
}

// Top returns the pair with minimum First value without removing it
func (m *MinHeap) Top() (Pair, bool) {
	if m.h.Len() == 0 {
		return Pair{}, false
	}
	return (*m.h)[0], true
}

// Pop removes and returns the pair with minimum First value
func (m *MinHeap) Pop() (Pair, bool) {
	if m.h.Len() == 0 {
		return Pair{}, false
	}
	return heap.Pop(m.h).(Pair), true
}

// Len returns the number of pairs in the heap
func (m *MinHeap) Len() int {
	return m.h.Len()
}

// IsEmpty returns true if the heap is empty
func (m *MinHeap) IsEmpty() bool {
	return m.h.Len() == 0
}
