@echo off
chcp 65001 >nul
title 工具管理系统 - 一键部署（环境检查 / 打包 / 安装 / 快捷方式）
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0deploy.ps1"
