@echo off
chcp 65001 >nul
title 工具管理系统 - 一键更新
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0update.ps1"
