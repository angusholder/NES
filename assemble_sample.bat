@echo off

.\cc65\bin\ca65.exe -t nes %1 -o %~dpn1.o && .\cc65\bin\ld65.exe -t nes -o %~dpn1.nes %~dpn1.o && del %~dpn1.o
