<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { fly } from "svelte/transition";
    import { quintOut } from "svelte/easing";
    function resizeObserver(node, callback) {
        const observer = new ResizeObserver(() => callback());
        observer.observe(node);

        return {
            destroy() {
                observer.disconnect();
            },
        };
    }
    let {
        id,
        url,
        image,
        pdf,
        bib,
        pages,
        summary,
        compactView = false,
        ondrop,
        ondragenter,
        ondragover,
        ondragleave,
        isSelected = false,
        isBeingDraggedOver = false,
    } = $props();
    let isHovered = $state(false);
    // Add these new state variables
    let paperElement;
    let distanceToRightEdge = $state(0);

    // Function to calculate distance
    function calculateDistance() {
        if (paperElement) {
            const paperRect = paperElement.getBoundingClientRect();
            const trayElement = document.querySelector(".tray");
            if (trayElement) {
                const trayRect = trayElement.getBoundingClientRect();
                distanceToRightEdge = trayRect.right - paperRect.right;
            }
        }
    }

    function handleMouseEnter(event) {
        // Check if the mouse enter event originated from the stack or paper elements
        if (
            event.target.closest(".paper") ||
            event.target.classList.contains("stack")
        ) {
            isHovered = true;
        }
    }

    function handleMouseLeave(event) {
        // Check if we're not entering the summary tray
        if (!event.relatedTarget?.closest(".summary-tray")) {
            isHovered = false;
        }
    }

    let showSummary = $derived(isHovered && summary);
    let year = bib?.issued
        ? bib.issued["date-parts"]?.[0]?.[0] || "Unknown year"
        : "Unknown year";
    let author = bib?.author
        ? bib.author.map((author) =>
              `${author.given || ""} ${author.family || ""}`.trim(),
          )
        : [];
    let image_url = $derived(image ? convertFileSrc(image) : null);
    let hasImage = $derived(image_url !== null);
    let page_count = $derived(trimSize(pages));

    function trimSize(x) {
        return Math.floor(Math.max(1, Math.min(15, Math.sqrt(x) / 1.5)));
    }

    function clampValue(value, limit) {
        if (Math.abs(value) > limit) {
            return value > 0 ? limit : -limit;
        }
        return value;
    }

    function stringToHash(str) {
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            const char = str.charCodeAt(i);
            hash = (hash << 5) - hash + char;
            hash |= 0; // Convert to 32bit integer
        }
        return hash;
    }

    function getComponentSeed(title) {
        const today = new Date();
        const dateSeed =
            today.getFullYear() * 10000 +
            (today.getMonth() + 1) * 100 +
            today.getDate();
        const titleHash = stringToHash(title);
        return dateSeed + titleHash; // Combine date and title hash
    }

    // Use a faster PRNG (e.g., Mulberry32)
    function mulberry32(a) {
        return function () {
            let t = (a += 0x6d2b79f5);
            t = Math.imul(t ^ (t >>> 15), t | 1);
            t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
            return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
        };
    }

    // Marsaglia polar method for generating normally distributed random numbers
    function getTRandomNumberMarsaglia(df, random) {
        let u, v, s;
        do {
            u = random() * 2 - 1;
            v = random() * 2 - 1;
            s = u * u + v * v;
        } while (s >= 1 || s === 0);

        let t = u * Math.sqrt(df * (s / (1 - s))); // Calculate t using one of the values

        t = clampValue(t, 7.5);
        return t;
    }

    // Create random number generator with today's seed
    const componentSeed = getComponentSeed(bib.title);
    const random = mulberry32(componentSeed);

    function getTRandomNumber(df) {
        return getTRandomNumberMarsaglia(df, random);
    }
</script>

{#if compactView}
    <div
        class="stack compact"
        {ondrop}
        {ondragenter}
        {ondragover}
        {ondragleave}
    >
        <h3>{bib.title}</h3>
        <p class="subinfo">{year}</p>
        <p class="subinfo">
            {author.join(", ")}
        </p>
    </div>
{:else}
    <div
        class="stack"
        bind:this={paperElement}
        use:resizeObserver={calculateDistance}
        {ondrop}
        {ondragenter}
        {ondragover}
        {ondragleave}
        onmouseenter={handleMouseEnter}
        onmouseleave={handleMouseLeave}
    >
        {#each { length: page_count } as _, i}
            <div
                class="paper"
                style="--random:{getTRandomNumber(3)}; --order:{i}"
            ></div>
        {/each}

        {#if showSummary}
            <div
                transition:fly={{
                    x: distanceToRightEdge < 250 ? 100 : -100,
                    opacity: 0.5,
                    delay: 0,
                    duration: 100,
                    easing: quintOut,
                }}
                class="summary-tray"
                class:left={distanceToRightEdge < 250}
            >
                <p class="info">
                    {summary}
                </p>
            </div>
        {/if}
        <div
            class="paper"
            style="--random:{getTRandomNumber(2)}; --order:{page_count};"
        >
            <div
                class="cover"
                class:selected={isSelected}
                class:dragover={isBeingDraggedOver}
            >
                {#if image}
                    <img
                        src={image_url}
                        alt={bib.title}
                        class="background-image"
                    />
                {/if}
                <div class="overlay" class:image-present={hasImage}>
                    <div class="text-content" class:top-positioned={!hasImage}>
                        <p class="subinfo">{year}</p>
                        <h3>{bib.title}</h3>
                        <p class="subinfo">
                            {author.join(", ")}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .stack {
        display: grid;
        place-items: center;
        position: relative;
        min-width: 10rem;
        aspect-ratio: 1/1.4;
    }
    .stack.compact {
        height: 3rem;
        width: 100%;
    }
    .paper {
        position: absolute;
        top: 0;
        display: grid;
        place-items: center;
        /* border: 1px solid rgba(0, 0, 0, 0.2); */
        border-radius: 5px;
        width: 12rem;
        aspect-ratio: 1/1.4;
        cursor: pointer;
        /* Functional props*/
        --rotation: calc(
            calc(30deg / -4) + calc(calc(30deg / 25)) * var(--order)
        );
        transform: rotate(calc(var(--random) * 2deg))
            translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
        transform-origin: center;
        transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
        background-color: white;
        --shadow: calc(var(--order) * 1px);
        box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
            rgba(0, 0, 0, 0.25);
        box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
            rgba(0, 0, 0, 0.25);
    }
    .paper:first-child {
        box-shadow: none;
    }
    .stack:hover > .paper {
        transform: rotate(calc(var(--random) * 1deg))
            translate(calc(var(--order) * -1px), calc(var(--order) * -1px));
        box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
            rgba(0, 0, 0, 0.25);
    }
    .cover {
        position: absolute;
        color: var(--hover);
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        border-radius: 5px;
        overflow-y: hidden;
    }
    .overlay {
        height: 100%;
        display: flex;
        flex-direction: column;
    }
    .overlay.image-present {
        justify-content: flex-end;
    }
    .text-content {
        padding: 0.5rem;
        background-color: white;
        z-index: 2;
        border-bottom-left-radius: 5px;
        border-bottom-right-radius: 5px;
    }
    .text-content.top-positioned {
        margin-top: 1rem;
    }
    /* Image styling */
    .background-image {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        border-top-left-radius: 5px;
        border-top-right-radius: 5px;
        object-fit: cover;
        z-index: -1;
    }

    .selected {
        border: 2px solid var(--accent-color-light);
    }
    .dragover {
        border: 2px solid var(--secondary-color-light);
    }
    .summary-tray {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        width: 200px;
        background-color: rgba(255, 255, 255, 0.9);
        padding: 0px 10px 10px 30px;
        border-radius: 5px;
        backdrop-filter: blur(5px);
        -webkit-backdrop-filter: blur(5px);
        opacity: 0.9;
        right: -240px;
        pointer-events: none; /* Prevent the tray from intercepting mouse events */
    }

    .summary-tray.left {
        right: auto;
        left: -240px;
        padding: 0px 30px 10px 10px;
    }

    .stack:hover {
        z-index: 20;
    }
</style>
