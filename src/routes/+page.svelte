<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="flex flex-col items-center justify-center text-center pt-[10vh] min-h-screen font-sans">
  <h1 class="text-center text-4xl mb-6">Welcome to Tauri + Svelte</h1>

  <div class="flex justify-center">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#747bff]" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#24c8db]" alt="Tauri Logo" />
    </a>
    <a href="https://kit.svelte.dev" target="_blank">
      <img src="/svelte.svg" class="h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#ff3e00]" alt="SvelteKit Logo" />
    </a>
  </div>
  <p class="mb-6">Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <form class="flex justify-center" onsubmit={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} class="mr-1 px-5 py-2 text-base font-medium rounded-lg border border-transparent bg-white text-gray-900 shadow-md transition-colors duration-250 outline-none hover:border-blue-600 dark:bg-gray-900/60 dark:text-white dark:hover:border-blue-400" />
    <button type="submit" class="px-5 py-2 text-base font-medium rounded-lg border border-transparent bg-white text-gray-900 shadow-md transition-colors duration-250 outline-none cursor-pointer hover:border-blue-600 active:border-blue-600 active:bg-gray-200 dark:bg-gray-900/60 dark:text-white dark:hover:border-blue-400 dark:active:bg-gray-900/40">Greet</button>
  </form>
  <p class="mt-4">{greetMsg}</p>
</main>

