#def f(n, m): return (n*(n+1) + m*(m-1)) // 2 + (n-1)*(m-1)
def f(n, m): return ((n+m)**2 - n - 3*m + 2) // 2


for m in range(1, 6+1):
    print([f(n, m) for n in range(1, 6+1)])
