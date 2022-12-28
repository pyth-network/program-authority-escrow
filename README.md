# program-authority-escrow

A minimalistic, stateless program to safely transfer a solana program from one upgrade authority to another one.

The way it works :
- The current authority uses Propose to transfer the authority of any program to a PDA of the escrow seeded by (current_authority, new_authority)
- Once the authority has been transferred two outcomes are possible : 
  - If the current authority calls Revert, the PDA will give the authority back to the current authority 
  - If the new authority calls Accept, the PDA will give the authority to the new authority

Basically, this program enforces that the new authority has signed before they accept the authority. 
This makes errors where we mistakenly transfer the authority to a key that we don't own reversible.