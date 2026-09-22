@echo off
rem THIS IS VERY IMPORTANT!!!
rem ==========================================
rem AUTHOR: M. SZUL
rem AI MODEL: GPT-5.6 Sol
rem TIMESTAMP: 2026-08-29 03:17:00 Europe/London
rem REASON FOR CREATION: Open the Darkstar Ubuntu SSH session from Windows with one double-click.
rem MECHANICS: Uses the Windows OpenSSH client and Serveo JumpHost syntax documented for private SSH forwarding.
rem SYSTEM PART: Darkstar management access
rem ARCHITECTURE FUNCTION: Administrative access only; not the Internet gateway path.
rem DEPENDENCIES/LINKS: Windows OpenSSH, serveo.net, Ubuntu reverse alias ds-cbms-8248.
rem TECH STACK: Windows cmd + OpenSSH.
rem LOCAL WORKSPACE: Windows client
rem GIT COMMIT: PENDING
rem GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-gateway
rem ==========================================

set "SSH=%SystemRoot%\System32\OpenSSH\ssh.exe"
if not exist "%SSH%" (
  echo ERROR: Windows OpenSSH client not found at:
  echo   %SSH%
  pause
  exit /b 1
)

title Darkstar SSH - ds-cbms-8248
"%SSH%" -tt -o ConnectTimeout=20 -o ServerAliveInterval=30 -o ServerAliveCountMax=3 -J serveo.net owner@ds-cbms-8248
set "RC=%ERRORLEVEL%"

if not "%RC%"=="0" (
  echo.
  echo Darkstar SSH ended with code %RC%.
  echo If the error says UNKNOWN port 65535 or banner timeout, check darkstar-serveo-tunnel.service on Ubuntu.
  pause
)
exit /b %RC%
