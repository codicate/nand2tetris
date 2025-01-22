// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

// sum = 0
@SUM
M=0

(LOOP)
// If R0 == 0, return
@R0
D=M
@RETURN
D;JEQ

// R0 -= 1
@R0
M=D-1

// sum += R1
@R1
D=M
@SUM
M=D+M

// loop
@LOOP
0;JMP

(RETURN)
@SUM
D=M
@R2
M=D

(END)
@END
0;JMP