# CSE-Icon AutoCoder - Sidebar Integration Guide

This guide explains how to integrate the CSE-Icon AutoCoder as a sidebar companion with Visual Studio Code and Visual Studio.

## Overview

CSE-Icon AutoCoder is designed to function as a sidebar companion that "snaps to" the side of your IDE, providing AI assistance while you code without requiring context switching between applications.

## Integration with Visual Studio Code

### Method 1: Side-by-Side Windows

1. Launch VS Code and CSE-Icon AutoCoder separately
2. Position the CSE-Icon AutoCoder window to the right of your VS Code window
3. Adjust the window sizes so they fit side by side on your screen
4. Use Windows 11's snap layout feature (hover over the maximize button) to automatically arrange the windows

### Method 2: Using the VS Code Extension (Recommended)

1. Install the CSE-Icon AutoCoder VS Code extension from the marketplace
2. Open VS Code and click on the CSE-Icon logo in the activity bar
3. The AutoCoder sidebar will appear docked to the side of VS Code
4. You can undock it by clicking the "Pop out" button in the sidebar header

## Integration with Visual Studio

### Method 1: Side-by-Side Windows

1. Launch Visual Studio and CSE-Icon AutoCoder separately
2. Position the CSE-Icon AutoCoder window to the right of your Visual Studio window
3. Adjust the window sizes so they fit side by side on your screen
4. Use Windows 11's snap layout feature to automatically arrange the windows

### Method 2: Using the Visual Studio Extension (Recommended)

1. Install the CSE-Icon AutoCoder Visual Studio extension from the marketplace
2. Open Visual Studio and go to View > CSE-Icon AutoCoder
3. The AutoCoder tool window will appear docked to the side of Visual Studio
4. You can customize its position in the Window > Dock menu

## Keyboard Shortcuts

- Toggle AutoCoder sidebar: `Ctrl+Shift+A`
- Focus on AutoCoder input: `Ctrl+Shift+I`
- Send current selection to AutoCoder: `Ctrl+Shift+S`
- Apply suggested code fix: `Ctrl+Shift+F`

## Configuration

You can configure the sidebar behavior in the settings:

1. Open CSE-Icon AutoCoder settings
2. Navigate to the "Integration" tab
3. Adjust the following options:
   - Default docking position (left/right)
   - Sidebar width
   - Auto-hide behavior
   - Startup integration mode

## Troubleshooting

If the sidebar integration isn't working correctly:

1. Ensure you have the latest version of CSE-Icon AutoCoder
2. Verify that the IDE extension is properly installed
3. Check if any other extensions are conflicting with the sidebar
4. Try restarting both the IDE and CSE-Icon AutoCoder
