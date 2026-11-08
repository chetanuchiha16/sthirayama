took the lock all the way,
used async move and used the tx again(wrong assumption that tx was sent, it was rx that was sent not tx)tx was used again because of the loop
did not clone the engine

fix 
put the whole loop inside the closure so tx isnt moved  again and again, clone the engine,  drop the lock as soon as you have chunk
