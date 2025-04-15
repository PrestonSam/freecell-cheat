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
Rank value of potential moves, compare them.

Let's just clone the game for the time being.
I'm planning to use a `Cons` model for the plan in future, but I need to get there first, so let's mutate the game.


## Bugs
None documented at present


## Planning

I could probably post the pick _into the column_ or so on to receive a chosen pick.
Although that's true, I wouldn't trust the pick, so I'd have to verify it a second time.
Although that's not ideal, it's not _awful_ either. It might be worth using this approach just so that I can at least have fully verified moves.
Even though there's a double verify.

The trouble I'm having is that the relationship involves _two_ entities.
That's where it's confusing.

I think my current exchange is pretty decent, but I still want to tighten it...
I think I need to blunder on with my current model, for the time being.


Alright we're onto scoring.
I need to determine 
