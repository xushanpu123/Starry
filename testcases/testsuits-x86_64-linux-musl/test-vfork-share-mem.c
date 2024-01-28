// Originally From: https://elixir.bootlin.com/glibc/glibc-2.38/source/posix/test-vfork.c
// Made it musl compatible

#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
// #include <error.h>
#include <errno.h>
#include <sys/wait.h>

// void __attribute_noinline__ noop (void);

#define NR	2	/* Exit code of the child.  */

/*
    The successful output is 
    Data originally: 10
    Child modified to 90
    Parent reading: 90
*/
int
main(void)
{
    int data=10;
    int status;
    pid_t pid;

    pid = vfork();
 
    if (pid == -1) {
        printf("create child process failed!\n");
        return -1;
    }
    if (pid == 0)
    {
        /* This will clobber the return pc from vfork in the parent on
     machines where it is stored on the stack, if vfork wasn't
     implemented correctly, */
        // noop ();
        printf("Data originally: %d\n",data);
        sleep(1);
        printf("Child modified to 90\n");
        data = 90;
        exit(0);
    }
    printf("Parent reading: %d\n",data);
    if (waitpid (0, &status, 0) != pid
        || !WIFEXITED (status) || WEXITSTATUS (status) != NR)
        exit (1); 

    return 0;
}

void
noop (void)
{
}
