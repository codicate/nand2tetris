// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.
(LOOP)
@KBD
D=M
@BLACKEN
D;JGT
@CLEAR
D;JEQ

(BLACKEN)
@color
M=-1
@DRAW
0;JMP

(CLEAR)
@color
M=0
@DRAW
0;JMP

(DRAW)
@8191
D=A
@i
M=D

(ITERATE)
@i
D=M
@SCREEN
D=D+A
@pixel
M=D

@color
D=M
@pixel
A=M
M=D

@i
D=M-1
M=D

@ITERATE
D;JGE

@LOOP
0;JMP