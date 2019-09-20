#! /usr/bin/env python3
import numpy as np
import matplotlib.pyplot as plt

sin = []
for line in open("sine.dat"):
    sin.append(float(line))
sin = np.array(sin)
print("len(sin) = {}".format(len(sin)))

saw = []
for line in open("saw.dat"):
    saw.append(float(line))
saw = np.array(saw)
print("len(saw) = {}".format(len(saw)))

square = []
for line in open("square.dat"):
    square.append(float(line))
square = np.array(square)
print("len(square) = {}".format(len(square)))

triangle = []
for line in open("triangle.dat"):
    triangle.append(float(line))
triangle = np.array(triangle)
print("len(triangle) = {}".format(len(triangle)))

plt.plot(sin)
plt.show()

plt.plot(saw)
plt.show()

plt.plot(square)
plt.show()

plt.plot(triangle)
plt.show()

