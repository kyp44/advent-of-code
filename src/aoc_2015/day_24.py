#! /usr/bin/env python3
""""
Run with -h for usage information.
"""
import argparse
from scipy.special import binom, factorial

# Parse command line arguments
parser = argparse.ArgumentParser(description="Calculates the number of partitions of a set with N elements into K subsets (i.e. the Stirling number of the second kind)")
parser.add_argument("n", metavar="N", type=int, help="Number of elements in the original set.")
parser.add_argument("k", metavar="K", type=int, help="Number of subsets into which to partition.")
args = parser.parse_args()

(n, k) = (args.n, args.k)

print(sum([(-1)**i * binom(k, i) * (k - i)**n for i in range(k+1)]) / factorial(k))
