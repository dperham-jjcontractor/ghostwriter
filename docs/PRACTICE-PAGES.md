# Practice pages

Four pages to test the coach before her first day and to re-check after any prompt change. She writes each one in her own handwriting (print first, then her normal hand), taps the face icon, and you score the reply with the five checks at the bottom. Run each page twice: replies vary a little.

To keep the last page and reply for review, add these two lines to `/home/root/.ghostwriter.toml` for the test week and restart the service (`systemctl restart ghostwriter`); remove them afterwards:

```toml
save_screenshot = "/home/root/last-page.png"
output_file = "/home/root/last-reply.txt"
```

## Page 1: training notes with a missing step

Write roughly this:

```
Returns training - Tues
- receipt = full refund to orig payment
- no receipt = merch credit, need ID
- 60 days? ask
- tags must be on
- final sale items no return
- classic awards points come off if returned
```

A good reply answers the "60 days?" question directly with the published policy (30 days for purchases after July 20, 2026), marked `(published policy)`, fills one or two gaps such as what counts as "worn" or exchanges without a receipt, gives two to four REMEMBER lines in its own words, and ends with one quiz question. It must not tell her to ask her manager.

## Page 2: shift recap with a customer name and a feeling

Write roughly this:

```
Sat shift
- floor set done by 10, tables looked good
- helped Mrs Patel find petite chatham in navy, she bought 2 + a sweater
- Karen said fold the cashmere with tissue
- markdowns took forever, missed lunch
- felt behind all day
```

A good reply names one specific thing that went well, gives two or three TRY NEXT SHIFT actions she can actually do (start markdowns earlier, ask for a buddy on markdown days, block lunch), and acknowledges the feeling in one line without a cliche. It must say "the customer", never the customer's name. It may use the coworker's first name.

## Page 3: floor set sketch with notes

Draw a rough rectangle for a front table with four boxes in it and write:

```
Front table plan
- new fall stripes on left
- cashmere stack middle
- scarves basket right?
- mannequin: stripe top + navy chatham + flats
- need signage for 30% sweaters
```

A good reply says what already works, suggests two or three specific adjustments (color order, a full outfit on the form, sign placement where it is seen from the aisle), and names one thing to compare against the visual guide.

## Page 4: a picture

On a fresh page write "draw me a front table layout with three zones" and tap. A good result: the tablet adds a page and draws a simple labelled layout with the pen, with nothing typed. Nothing should appear on the page she wrote on.

## Scoring: five checks per reply

| Check | Pass when |
|---|---|
| Header | First line is `-- COACH --` |
| Length | Under 700 characters, typed in about 7 seconds or less |
| Readable | No missing characters, no stray symbols, no markdown |
| Privacy | No customer name on page 2 |
| Useful | At least one fact or action she could not have written herself, and no "ask your manager" |

If a check fails twice on the same page, edit `prompts/coach.txt` or `prompts/store-context.txt`, rebuild with `.\tools\make-prompt-json.ps1`, copy the file to the tablet, and run the page again. Keep the reply to page 1 from the first day; it is the baseline for judging later changes.
