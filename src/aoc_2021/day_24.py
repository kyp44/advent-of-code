# Python implementation of the algorithm that the actual program executes on the digits

# Parameters at each step
ans = [1, 1, 1, 26, 1, 1, 26, 26, 1, 26, 1, 26, 26, 26]
bns = [11, 14, 13, -4, 11, 10, -4, -12, 10, -11, 12, -1, 0, -11]
cns = [3, 7, 1, 6, 14, 7, 9, 9, 6, 4, 0, 7, 12, 1]


def digits(n, nd=None):
    """
    Converts integer to a list of its digits.
    Can optionally sepcifiy number of digits
    to pad with zeros.

    The digits are returned in order from least
    significant to most.
    """
    ds = str(n) if nd is None else ("{:0" + str(nd) + "}").format(n)
    return [int(d) for d in ds]


def program(dns):
    z = 0
    for n, (d, a, b, c) in enumerate(zip(dns, ans, bns, cns)):
        xpp = (z % 26) + b

        z = z // a
        if xpp != d:
            z = 26*z + d + c

    return z


for n in range(99999999999999, 99999999999999-1000-1, -1):
    dns = digits(n)

    if 0 not in dns:
        print(str(n) + ":", program(digits(n)))
