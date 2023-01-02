/***************************************************************************
*             __________               __   ___.
*   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
*   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
*   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
*   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
*                     \/            \/     \/    \/            \/
* $Id$
*
* This is a memory allocator designed to provide reasonable management of free
* space and fast access to allocated data. More than one allocator can be used
* at a time by initializing multiple contexts.
*
* Copyright (C) 2009 Andrew Mahone
* Copyright (C) 2011 Thomas Martitz
*
* This program is free software; you can redistribute it and/or
* modify it under the terms of the GNU General Public License
* as published by the Free Software Foundation; either version 2
* of the License, or (at your option) any later version.
*
* This software is distributed on an "AS IS" basis, WITHOUT WARRANTY OF ANY
* KIND, either express or implied.
*
****************************************************************************/
#ifndef _BUFLIB_MEMPOOL_H_
#define _BUFLIB_MEMPOOL_H_

#ifndef _BUFLIB_H_
# error "include buflib.h instead"
#endif

union buflib_data
{
    intptr_t val;                 /* length of the block in n*sizeof(union buflib_data).
                                     Includes buflib metadata overhead. A negative value
                                     indicates block is unallocated */
    volatile unsigned pincount;   /* number of pins */
    struct buflib_callbacks* ops; /* callback functions for move and shrink. Can be NULL */
    char* alloc;                  /* start of allocated memory area */
    union buflib_data *handle;    /* pointer to entry in the handle table.
                                     Used during compaction for fast lookup */
};

struct buflib_context
{
    union buflib_data *handle_table;
    union buflib_data *first_free_handle;
    union buflib_data *last_handle;
    union buflib_data *buf_start;
    union buflib_data *alloc_end;
    bool compact;
};

#define BUFLIB_ALLOC_OVERHEAD (4 * sizeof(union buflib_data))

#ifndef BUFLIB_DEBUG_GET_DATA
static inline void *buflib_get_data(struct buflib_context *ctx, int handle)
{
    return (void *)ctx->handle_table[-handle].alloc;
}
#endif

#endif /* _BUFLIB_MEMPOOL_H_ */
