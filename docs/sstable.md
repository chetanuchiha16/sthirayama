count the number of entries on the skiplist, when it reaches a limit, start the write to sstable process

traverse the skiplist, serialize the data get the len, write the len and data as bytes, 

after every 4kb start new block note down the highest key, the len and offset of each block