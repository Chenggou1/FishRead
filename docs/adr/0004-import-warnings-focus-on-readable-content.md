# Import warnings focus on readable content

FishRead treats text-bearing EPUB spine items as importable reading content and silently skips spine items that produce no readable text, such as covers and title-art image pages. Import warnings are reserved for non-fatal degradations that may affect readable content or user-visible metadata, so expected omissions of auxiliary image-only spine items do not make successful imports look risky.
