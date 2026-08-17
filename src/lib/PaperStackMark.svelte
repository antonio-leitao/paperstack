<script lang="ts">
  /* The PaperStack mark.

     One geometry, two stroke treatments. The two files in design_system/ differ
     only in their linework — the paths are identical — so they are one component
     with a `bold` prop rather than two copies of 40 paths that can drift apart.

     Light is the default because that is what in-app placements want: an empty
     state or an about panel sits on --paper at a comfortable size, where the
     grey linework reads as drawn paper. Bold is the app icon's treatment, kept
     here for anywhere the mark has to hold up small or against a strong colour.

     The viewBox is the drawing's measured content box, not the 1000-unit canvas
     it was authored in. The artwork sits off-centre in that canvas, so a naive
     `0 0 1000 1000` leaves the mark low and left with dead space around it. */

  let { size = 48, bold = false, title = "" }: {
    size?: number;
    bold?: boolean;
    title?: string;
  } = $props();

  const W = 765;
  const H = 657;

  /* The sheet under each curled corner: drawn as the whole flat top face
     and then covered by the top sheet, so only the wedge the curl lifts
     away from shows. Without it the notch is empty and the stack reads as
     hollow. Its exposed back edge is the short segment appended to each
     structure path, ending where it meets the curl. */
  const BELOW = "#e8e6e0";

  let structure = $derived(bold ? "#20201e" : "#b8b6af");
  let detail = $derived(bold ? "#20201e" : "#aaa8a1");
  let sw = $derived(bold ? 4.5 : 2.5);
  let dw = $derived(bold ? 4 : 2.5);
</script>

<svg
  viewBox="117 180 {W} {H}"
  width={(size * W) / H}
  height={size}
  shape-rendering="geometricPrecision"
  role={title ? "img" : "presentation"}
  aria-hidden={title ? undefined : "true"}
  aria-label={title || undefined}
>
  <!-- Tall centre stack first, so the shorter stacks overlap it from the front. -->
  <g>
    <path fill="#e8e6e0" d="M400 275 580 319 580 800 400 750Z" />
    <path fill="#f5f4f1" d="M580 319 718 250 718 731 580 800Z" />
    <path fill={BELOW} d="M400 275 538 206 718 250 580 319Z" />
    <path fill="#ffffff" d="M400 275 545 192C601 204 661 211 718 183 690 234 635 283 580 319Z" />
    <path fill="#efeee9" d="M651 205C678 202 700 194 718 183 690 234 635 283 580 319 624 277 648 235 651 205Z" />
    <g fill="none" stroke={structure} stroke-width={sw} stroke-linecap="round" stroke-linejoin="round">
      <path d="M400 275 580 319 718 250M400 275 545 192C601 204 661 211 718 183M400 275V750L580 800 718 731V250M580 319V800" />
      <path d="M718 183C690 234 635 283 580 319M651 205C648 244 621 284 580 319M718 250 676 240" />
    </g>
    <g fill="none" stroke={detail} stroke-width={dw} stroke-linecap="round" stroke-linejoin="round">
      <path d="M425 381 484 396M641 376 692 351M442 522 497 536M622 493 674 468" />
      <path d="M516 628 580 646 610 631" />
    </g>
  </g>

  <g>
    <path fill="#e8e6e0" d="M120 430 290 475 290 810 120 760Z" />
    <path fill="#f5f4f1" d="M290 475 420 410 420 750 290 810Z" />
    <path fill={BELOW} d="M120 430 250 365 420 410 290 475Z" />
    <path fill="#ffffff" d="M120 430 260 355C310 370 365 377 420 350 393 401 341 445 290 475Z" />
    <path fill="#efeee9" d="M359 371C382 369 403 361 420 350 393 401 341 445 290 475 329 437 354 399 359 371Z" />
    <g fill="none" stroke={structure} stroke-width={sw} stroke-linecap="round" stroke-linejoin="round">
      <path d="M120 430 290 475 420 410M120 430 260 355C310 370 365 377 420 350M120 430V760L290 810 420 750V410M290 475V810" />
      <path d="M420 350C393 401 341 445 290 475M359 371C355 405 330 443 290 475M420 410 384 400" />
    </g>
    <g fill="none" stroke={detail} stroke-width={dw} stroke-linecap="round" stroke-linejoin="round">
      <path d="M143 551 201 566M333 548 384 523M177 663 231 677" />
      <path d="M246 720 290 733 322 718" />
    </g>
  </g>

  <g>
    <path fill="#e8e6e0" d="M620 560 765 600 765 835 620 795Z" />
    <path fill="#f5f4f1" d="M765 600 880 545 880 785 765 835Z" />
    <path fill={BELOW} d="M620 560 735 505 880 545 765 600Z" />
    <path fill="#ffffff" d="M620 560 740 500C785 513 834 515 880 490 855 537 812 574 765 600Z" />
    <path fill="#efeee9" d="M829 508C848 506 866 499 880 490 855 537 812 574 765 600 800 568 824 536 829 508Z" />
    <g fill="none" stroke={structure} stroke-width={sw} stroke-linecap="round" stroke-linejoin="round">
      <path d="M620 560 765 600 880 545M620 560 740 500C785 513 834 515 880 490M620 560V795L765 835 880 785V545M765 600V835" />
      <path d="M880 490C855 537 812 574 765 600M829 508C825 538 802 569 765 600M880 545 847 536" />
    </g>
    <g fill="none" stroke={detail} stroke-width={dw} stroke-linecap="round" stroke-linejoin="round">
      <path d="M645 688 698 702M808 675 851 654M654 758 703 772" />
      <path d="M728 743 765 754 792 741" />
    </g>
  </g>
</svg>

<style>
  svg {
    display: block;
  }
</style>
