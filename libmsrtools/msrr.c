/*
 * msrr.c
 *
 * Utility to read an MSR.
 */

#include <errno.h>
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
#include <inttypes.h>
#include <sys/types.h>
#include <dirent.h>
#include <ctype.h>

/* Number of decimal digits for a certain number of bits */
/* (int) ceil(log(2^n)/log(10)) */
int decdigits[] = {
        1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 4, 5, 5,
        5, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10,
        10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 15,
        15, 15, 16, 16, 16, 16, 17, 17, 17, 18, 18, 18, 19, 19, 19, 19,
        20
};

#define mo_hex  0x01
#define mo_dec  0x02
#define mo_chx  0x06

const char *program;

uint64_t rdmsr_on_cpu(uint32_t reg, int cpu);

/* filter out ".", "..", "microcode" in /dev/cpu */
int dir_filter(const struct dirent *dirp) {
    if (isdigit(dirp->d_name[0]))
        return 1;
    else
        return 0;
}

unsigned int highbit = 63, lowbit = 0;
int mode = mo_hex;

uint64_t rdmsr_on_cpu(uint32_t reg, int cpu)
{
    uint64_t data;
    int fd;
    char msr_file_name[64];
    unsigned int bits;

    sprintf(msr_file_name, "/dev/cpu/%d/msr", cpu);
    fd = open(msr_file_name, O_RDONLY);
    if (fd < 0) {
        if (errno == ENXIO) {
            fprintf(stderr, "rdmsr: No CPU %d\n", cpu);
            exit(2);
        } else if (errno == EIO) {
            fprintf(stderr, "rdmsr: CPU %d doesn't support MSRs\n",
                    cpu);
            exit(3);
        } else {
            perror("rdmsr: open");
            exit(127);
        }
    }

    if (pread(fd, &data, sizeof data, reg) != sizeof data) {
        if (errno == EIO) {
            fprintf(stderr, "rdmsr: CPU %d cannot read "
                            "MSR 0x%08"PRIx32"\n",
                    cpu, reg);
            exit(4);
        } else {
            perror("rdmsr: pread");
            exit(127);
        }
    }

    close(fd);

    bits = highbit - lowbit + 1;
    if (bits < 64) {
        /* Show only part of register */
        data >>= lowbit;
        data &= (1ULL << bits) - 1;
    }

    return data;
}
