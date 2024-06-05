
// include/mylib.h

#ifndef MYLIB_H
#define MYLIB_H

#ifdef _WIN32
    #ifdef MYLIB_EXPORTS
        #define MYLIB_API __declspec(dllexport)
    #else
        #define MYLIB_API __declspec(dllimport)
    #endif
#else
    #define MYLIB_API
#endif

// Function prototype for the library function
MYLIB_API void my_function();

#endif // MYLIB_H

