#pragma once

#include <Windows.h>

// true if what starts with with
bool startswith(const char *what, const char *with);

// fills name with pid name, true on success
bool getprocname(DWORD processID, char *name, DWORD namelen);

// finds the process starting with name startwith
// updates name with full process name
// returns pid or 0 on not found
DWORD findproc(const char *startwith, char *name, DWORD namelen);
