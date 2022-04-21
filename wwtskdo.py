import tskit

x = 50
y = 60
l = 100

t = tskit.TableCollection(l)

# Add our nodes

t.nodes.add_row(0, 2)
t.nodes.add_row(0, 2)
t.nodes.add_row(0, 1)
t.nodes.add_row(0, 1)
t.nodes.add_row(0, 0)
t.nodes.add_row(0, 0)

t.edges.add_row(0, x, 0, 2)

t.edges.add_row(x, l, 1, 2)
t.edges.add_row(0, l, 1, 3)

t.edges.add_row(0, y, 2, 5)

t.edges.add_row(0, l, 3, 4)
t.edges.add_row(y, l, 3, 5)

print("before")
print(t)

t.sort()
samples = [4, 5]
# samples = [5, 4]

idmap = t.simplify(samples)

print("after first")
print(t)

parents = [idmap[i] for i in samples]

print(parents)

# Add the next 4
samples = []
for i in range(4):
    samples.append(t.nodes.add_row(0, -1))

# new breakpoints
a = 25
b = 75
c = 10
d = 90

print(samples)
print(t.nodes.time[samples])
print(t.nodes.time[parents])

t.edges.add_row(a, l, 0, 4)
t.edges.add_row(b, l, 0, 6)
t.edges.add_row(0, d, 1, 5)
t.edges.add_row(c, d, 1, 7)

t.sort()

idmap = t.simplify(samples)

print(t)
