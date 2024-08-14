// src/stores.js
import { writable } from 'svelte/store';

// Create a writable store that holds a string
export const info = writable('');
