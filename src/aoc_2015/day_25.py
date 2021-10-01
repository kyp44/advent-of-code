#! /usr/bin/env python3
""""
Run with -h for usage information.
"""
import argparse
import itertools as it

# Parse command line arguments
parser = argparse.ArgumentParser(description="Tests modulo arithmetic compared with separate steps.")
parser.add_argument("n", metavar="N", type=int, help="Number of steps to calculate.")
args = parser.parse_args()

init = 20151125
m = 252533
p = 33554393
print("init:", init, "m:", m, "p:", p)

def steps(n) :
    x = init
    for i in range(n) :
        x = (m * x) % p
    return x

def modulo(n) :
    return (m**n * init) % p

print(steps(args.n), modulo(args.n))

"""
for n in it.count() :
    x = modulo(n)
    if x in vals :
        print("n:", n, "x:", x)
        break
    vals.append(x)
"""
