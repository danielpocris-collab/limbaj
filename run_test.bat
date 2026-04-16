@echo off
ngc_gen1.exe tests\programs\exit42.ng
echo EXIT:%ERRORLEVEL%
dir output.exe
