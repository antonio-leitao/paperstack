<script>
    export let author;
    export let id;
    export let slug;
    export let title;
    export let url;
    export let year;
    import { invoke } from "@tauri-apps/api/tauri";

    async function loadImage() {
        try {
            const base64 = await invoke("load_image", {
                key: `${id}`,
            });
            return `data:image/jpeg;base64,${base64}`;
        } catch (error) {
            console.error("Failed to load image:", error);
            throw error;
        }
    }
</script>

<div class="card">
    <div class="image-container">
        {#await loadImage()}
            <div class="skeleton">
                <!-- SVG placeholder for skeleton image -->
                <svg
                    width="100%"
                    height="100%"
                    viewBox="0 0 300 200"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <rect width="100%" height="100%" fill="#e0e0e0" />
                    <circle cx="150" cy="100" r="50" fill="#c0c0c0" />
                    <rect
                        x="100"
                        y="160"
                        width="100"
                        height="20"
                        fill="#c0c0c0"
                    />
                </svg>
            </div>
        {:then imageSrc}
            <img src={imageSrc} alt="Cover image" />
        {:catch error}
            <p>Error loading image: {error.message}</p>
        {/await}
    </div>
    <div class="details">
        <div class="title">{title}</div>
        <div class="authors">
            <div class="author">{author}</div>
        </div>
    </div>
</div>

<style>
    .card {
        display: flex;
        align-items: center;
        border-radius: 8px;
        padding: 16px;
        max-width: 600px;
        margin: 0 auto;
        background-color: white;
        transition: all 0.3s cubic-bezier(0, 0.55, 0.45, 1); /* Slower transition for the background */
        margin-bottom: 0.75rem;
    }
    .card:hover {
        box-shadow:
            0 4px 6px -1px rgb(0 0 0 / 0.1),
            0 2px 4px -2px rgb(0 0 0 / 0.1);
    }

    .image-container {
        flex-shrink: 0;
        width: 130px; /* Adjusted width to account for the crop */
        height: 150px;
        overflow: hidden;
        border-radius: 8px;
        margin-left: -10px; /* Crop by 10px on the left */
        margin-right: -10px; /* Crop by 10px on the right */
        /*box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05); /* Small shadow under the image */
    }

    .image-container img {
        width: calc(
            100% + 20px
        ); /* Add back the 20px to compensate for the crop */
        height: 100%;
        object-fit: cover;
        margin-left: -10px; /* Center the cropped image */
    }

    .details {
        margin-left: 16px;
        flex-grow: 1;
    }

    .title {
        font-size: 1.2rem;
        font-weight: bold;
        color: #333;
        margin-bottom: 8px;
    }

    .authors {
        font-size: 0.9rem;
        color: #555;
    }

    .author {
        margin-bottom: 4px;
    }
</style>
