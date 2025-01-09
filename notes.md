### COMMON TERMS

Foundation|Description
---|---
Foundation | the piles where cards are built in order, typically by suit
Playable pair | Two cards that can be stacked one upon the other
Tableau | The main playing area with multiple columns of cards
Stock | The pile of remaining cards to draw from
Waste | the discard pile for cards drawn from the Stock
Cascade | A column of cards in the Tableau
Build | to place cards in a specific sequence, usually alternating colours or descending order
Move | The act of transferring a card or stack to another location
Fan | A spread of overlapping cards
Reserve | A special holding area for cards (e.g. FreeCell's free cells)
Suit sequence | A group of cards in ascending order of the same suit
Playable card | A card eligable for movement based on the rules
Empty column | A vacant tableau column that can often hold a king or other cards.





### SCORING RULES

Goals  
Spot playable pairs (including a pair of a card and a stack)  
I'll make the assumption that subdividing a stack is only worth it when you're trying to move a large stack.  
Getting an empty column is very valuable.  
Disincentivise filling the reserve  
You need to figure out how to tell the algorithm to spot cards hidden under other cards  
  ^ this would be approximately solved using brute force BFS, although that's not a very useful solution.  

I think the algorithm should largely be a BFS, where you explore every possible next move, then sort descending by score  
Then you take x moves from the head of the sorted vec, repeating the process.  
I think the algorithm should also plan ahead by calculating a set of moves rather than a single one.  
It'd be the next x moves, then calculate  
And it'd be a random search  
Then I can configure X depending on performance and so on.  
Alright  




## TODO

Use pointers to the cards rather than storing them.  
I don't exactly know how it'll be done maybe I should use an Rc.  
We'll get there...
I need to add rules for what's valid to put where
I realise the rules are different in each case FreeCell is a lot more complex than I really considered it to be

## Bugs
