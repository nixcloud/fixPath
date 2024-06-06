// include/delayedlib.h

#ifndef DELAYEDLIB_H
#define DELAYEDLIB_H

#ifdef _WIN32
    #ifdef DELAYEDLIB_EXPORTS
        #define DELAYEDLIB_API __declspec(dllexport)
    #else
        #define DELAYEDLIB_API __declspec(dllimport)
    #endif
#else
    #define DELAYEDLIB_API
#endif

// Function prototype for the delayed library function
DELAYEDLIB_API void delayed_function();

#endif // DELAYEDLIB_H