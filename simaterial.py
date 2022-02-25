from dataclasses import dataclass

import typing
import heapq
# import numpy as np


@dataclass
class Segment:
    left: int
    right: int
    node: int

    def __post_init__(self):
        assert self.left < self.right

    def __lt__(self, other):
        return self.left < other.left


@dataclass
class Node:
    time: int
    flags: int = 0


@dataclass
class Edge:
    left: int
    right: int
    parent: int
    child: int


def simplify(S: typing.List[int], N: typing.List[Node], E: typing.List[Edge], L: int):

    A = [[] for _ in range(len(N))]
    Q = []

    No = []
    Eo = []

    for u in S:
        No.append(Node(N[u].time, 1))
        A[u] = [Segment(0, L, len(No) - 1)]

    for input_node in range(len(N)):
        u = len(N) - input_node - 1
        assert len(Q) == 0
        for e in [e for e in E if e.parent == u]:
            for x in A[e.child]:
                if x.right > e.left and e.right > x.left:
                    y = Segment(max(x.left, e.left), min(x.right, e.right), x.node)
                    heapq.heappush(Q, y)

        output_node = -1
        while len(Q) > 0:
            left_position = Q[0].left
            right_position = L
            X = []  # the overlaps
            while len(Q) > 0 and Q[0].left == left_position:
                x = heapq.heappop(Q)
                X.append(x)
                right_position = min(right_position, x.right)
            if len(Q) > 0:
                right_position = min(right_position, Q[0].left)
            if len(X) == 1:
                x = X[0]
                alpha = x
                if len(Q) > 0 and Q[0].left < x.right:
                    alpha = Segment(x.left, Q[0].left, x.node)
                    x.left = Q[0].left
                    heapq.heappush(Q, x)
            else:
                if output_node == -1:
                    No.append(Node(N[u].time))
                    output_node = len(No) - 1
                alpha = Segment(left_position, right_position, output_node)
                for x in X:
                    Eo.append(Edge(left_position, right_position, output_node, x.node))
                    if x.right > right_position:
                        x.left = right_position
                        heapq.heappush(Q, x)

            A[u].append(alpha)

    # Sort, but do not squash, the output edges
    Eo.sort(key=lambda e: (e.parent, e.child, e.right, e.left))
    return No, Eo, A


if __name__ == "__main__":
    N = []
    E = []

    N.append(Node(2))
    N.append(Node(2))
    N.append(Node(1))
    N.append(Node(1))
    N.append(Node(0))
    N.append(Node(0))

    x = 50
    y = 60
    L = 100

    E.append(Edge(0, x, 0, 2))
    E.append(Edge(x, L, 1, 2))
    E.append(Edge(0, L, 1, 3))

    E.append(Edge(0, y, 2, 5))

    E.append(Edge(0, L, 3, 4))
    E.append(Edge(y, L, 3, 5))

    print(E)

    samples = [4, 5]

    assert len(N) == 6
    assert len(E) == 6

    No, Eo, A = simplify(samples, N, E, L)

    print(No)
    for i, e in enumerate(Eo):
        print(i, e)
    print(A)
