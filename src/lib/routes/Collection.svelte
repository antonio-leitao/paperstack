<script>
  import Paper from "../components/Paper.svelte";
  import PopFuzzySearch from "../components/PopFuzzySearch.svelte";
  import { Edit3, Files, Link, Wifi, WifiOff, Trash2 } from "lucide-svelte";
  let papers = [
    {
      paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
      url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
      title: "Construction of the Literature Graph in Semantic Scholar",
      abstract:
        "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
      year: 2018,
      citationCount: 296,
      openAccessPdf: {
        url: "https://www.aclweb.org/anthology/N18-3011.pdf",
        status: "HYBRID",
      },
      citationStyles: {
        bibtex:
          "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
      },
      authors: [
        { authorId: "145585097", name: "Waleed Ammar" },
        { authorId: "3458736", name: "Dirk Groeneveld" },
        { authorId: "1857797", name: "Chandra Bhagavatula" },
        { authorId: "46181066", name: "Iz Beltagy" },
        { authorId: "46230609", name: "Miles Crawford" },
        { authorId: "145612610", name: "Doug Downey" },
        { authorId: "38092776", name: "Jason Dunkelberger" },
        { authorId: "143718836", name: "Ahmed Elgohary" },
        { authorId: "46411828", name: "Sergey Feldman" },
        { authorId: "4480314", name: "Vu A. Ha" },
        { authorId: "143967880", name: "Rodney Michael Kinney" },
      ],
    },
    {
      paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
      url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
      title: "Construction of the Literature Graph in Semantic Scholar",
      abstract:
        "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
      year: 2018,
      citationCount: 296,
      openAccessPdf: {
        url: "https://www.aclweb.org/anthology/N18-3011.pdf",
        status: "HYBRID",
      },
      citationStyles: {
        bibtex:
          "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
      },
      authors: [
        { authorId: "145585097", name: "Waleed Ammar" },
        { authorId: "3458736", name: "Dirk Groeneveld" },
        { authorId: "1857797", name: "Chandra Bhagavatula" },
        { authorId: "46181066", name: "Iz Beltagy" },
        { authorId: "46230609", name: "Miles Crawford" },
        { authorId: "145612610", name: "Doug Downey" },
        { authorId: "38092776", name: "Jason Dunkelberger" },
        { authorId: "143718836", name: "Ahmed Elgohary" },
        { authorId: "46411828", name: "Sergey Feldman" },
        { authorId: "4480314", name: "Vu A. Ha" },
        { authorId: "143967880", name: "Rodney Michael Kinney" },
      ],
    },
    {
      paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
      url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
      title: "Construction of the Literature Graph in Semantic Scholar",
      abstract:
        "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
      year: 2018,
      citationCount: 296,
      openAccessPdf: {
        url: "https://www.aclweb.org/anthology/N18-3011.pdf",
        status: "HYBRID",
      },
      citationStyles: {
        bibtex:
          "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
      },
      authors: [
        { authorId: "145585097", name: "Waleed Ammar" },
        { authorId: "3458736", name: "Dirk Groeneveld" },
        { authorId: "1857797", name: "Chandra Bhagavatula" },
        { authorId: "46181066", name: "Iz Beltagy" },
        { authorId: "46230609", name: "Miles Crawford" },
        { authorId: "145612610", name: "Doug Downey" },
        { authorId: "38092776", name: "Jason Dunkelberger" },
        { authorId: "143718836", name: "Ahmed Elgohary" },
        { authorId: "46411828", name: "Sergey Feldman" },
        { authorId: "4480314", name: "Vu A. Ha" },
        { authorId: "143967880", name: "Rodney Michael Kinney" },
      ],
    },
    {
      paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
      url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
      title: "Construction of the Literature Graph in Semantic Scholar",
      abstract:
        "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
      year: 2018,
      citationCount: 296,
      openAccessPdf: {
        url: "https://www.aclweb.org/anthology/N18-3011.pdf",
        status: "HYBRID",
      },
      citationStyles: {
        bibtex:
          "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
      },
      authors: [
        { authorId: "145585097", name: "Waleed Ammar" },
        { authorId: "3458736", name: "Dirk Groeneveld" },
        { authorId: "1857797", name: "Chandra Bhagavatula" },
        { authorId: "46181066", name: "Iz Beltagy" },
        { authorId: "46230609", name: "Miles Crawford" },
        { authorId: "145612610", name: "Doug Downey" },
        { authorId: "38092776", name: "Jason Dunkelberger" },
        { authorId: "143718836", name: "Ahmed Elgohary" },
        { authorId: "46411828", name: "Sergey Feldman" },
        { authorId: "4480314", name: "Vu A. Ha" },
        { authorId: "143967880", name: "Rodney Michael Kinney" },
      ],
    },
    {
      paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
      url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
      title: "Construction of the Literature Graph in Semantic Scholar",
      abstract:
        "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
      year: 2018,
      citationCount: 296,
      openAccessPdf: {
        url: "https://www.aclweb.org/anthology/N18-3011.pdf",
        status: "HYBRID",
      },
      citationStyles: {
        bibtex:
          "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
      },
      authors: [
        { authorId: "145585097", name: "Waleed Ammar" },
        { authorId: "3458736", name: "Dirk Groeneveld" },
        { authorId: "1857797", name: "Chandra Bhagavatula" },
        { authorId: "46181066", name: "Iz Beltagy" },
        { authorId: "46230609", name: "Miles Crawford" },
        { authorId: "145612610", name: "Doug Downey" },
        { authorId: "38092776", name: "Jason Dunkelberger" },
        { authorId: "143718836", name: "Ahmed Elgohary" },
        { authorId: "46411828", name: "Sergey Feldman" },
        { authorId: "4480314", name: "Vu A. Ha" },
        { authorId: "143967880", name: "Rodney Michael Kinney" },
      ],
    },
    {
      paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
      url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
      title: "Construction of the Literature Graph in Semantic Scholar",
      abstract:
        "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
      year: 2018,
      citationCount: 296,
      openAccessPdf: {
        url: "https://www.aclweb.org/anthology/N18-3011.pdf",
        status: "HYBRID",
      },
      citationStyles: {
        bibtex:
          "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
      },
      authors: [
        { authorId: "145585097", name: "Waleed Ammar" },
        { authorId: "3458736", name: "Dirk Groeneveld" },
        { authorId: "1857797", name: "Chandra Bhagavatula" },
        { authorId: "46181066", name: "Iz Beltagy" },
        { authorId: "46230609", name: "Miles Crawford" },
        { authorId: "145612610", name: "Doug Downey" },
        { authorId: "38092776", name: "Jason Dunkelberger" },
        { authorId: "143718836", name: "Ahmed Elgohary" },
        { authorId: "46411828", name: "Sergey Feldman" },
        { authorId: "4480314", name: "Vu A. Ha" },
        { authorId: "143967880", name: "Rodney Michael Kinney" },
      ],
    },
  ];
  export let params = {};

  function getTRandomNumber(df) {
    var z1 = Math.random() * 2 - 1; // uniform(-1, 1)
    var z2 = Math.random() * 2 - 1; // uniform(-1, 1)

    var s = z1 * z1 + z2 * z2;
    while (s === 0 || s > 1) {
      z1 = Math.random() * 2 - 1; // uniform(-1, 1)
      z2 = Math.random() * 2 - 1; // uniform(-1, 1)
      s = z1 * z1 + z2 * z2;
    }
    var t = z1 * Math.sqrt(df * (s / (1 - s)));
    return t;
  }

  let N = papers.length + 1;
  let visible = false;
  function onPick(e) {
    hiddenSearch = true;
    selectedOption = e.detail;
    console.log(selectedOption);
  }
  let options = [
    { label: "Example", link: "/" },
    { label: "Search with fuzzy", link: "/also" },
    { label: "Bolds the text", link: "/in" },
    { label: "Other example", link: "/the-link" },
  ];
  let selectedOption;
  let hiddenSearch = true;

  //BUTTON HANDLING
  function copyBib(e) {
    console.log(e.detail);
    console.log(hiddenSearch);
    hiddenSearch = false;
  }
</script>

<PopFuzzySearch
  {options}
  bind:hidden={hiddenSearch}
  on:pick={onPick}
  keys={["label", "link"]}
>
  <h3 slot="title">Move paper to which stack:</h3>
</PopFuzzySearch>
<div class="header">
  <h2>{params.stackId}</h2>

  <p>Selected: {JSON.stringify(selectedOption)}</p>

  <div class="buttons">
    <div class="icon">
      <Edit3 />
    </div>
    <div class="icon">
      <Files />
    </div>
    <div class="icon">
      {#if visible}
        <Wifi />
      {:else}
        <WifiOff />
      {/if}
    </div>
    <div class="icon">
      <Link />
    </div>
  </div>
</div>

{#each papers as paper, i}
  {#if i == 0}
    <div
      class="cards"
      style="--random:{getTRandomNumber(2)};--order:{i + 1};height: {90 *
        (N - i)}vh;"
    >
      <div class="card">
        <Paper {...paper} on:update={copyBib} />
      </div>
    </div>
  {:else}
    <div
      class="cards"
      style="--random:{getTRandomNumber(3)};--order:{i + 1};height: {90 *
        (N - i)}vh;margin-top: {90 * (i - N)}vh;"
    >
      <div class="card">
        <Paper {...paper} on:update={copyBib} />
      </div>
    </div>
  {/if}
{/each}

<div class="footer">
  <div class="buttons">
    <div class="icon trash">
      <Trash2 strokeWidth="1" size="42" />
    </div>
  </div>
</div>

<style>
  .header {
    height: 30vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .footer {
    position: relative;
    z-index: 1;
    height: 50vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cards {
    width: max(70%, 30rem);
  }
  .card {
    top: 2rem;
    position: sticky;
    display: flex;
    align-items: left;
    flex-direction: column;
    border-radius: 1rem;
    background-position: center;
    background-color: white;
    /* border: 1px lightgray solid; */
    height: 90vh;

    padding: 3rem;
    padding-top: 10rem;
    --shadow: calc(var(--order) * 3px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 50px)
      rgba(0, 0, 0, 0.25);
    transform: rotate(calc(var(--random) * 2deg))
      translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
  }

  .buttons {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
    color: gray;
  }
  .icon {
    margin: 2rem;
    cursor: pointer;
  }
  .icon:hover {
    color: black;
  }
  .trash:hover {
    color: red;
  }
</style>
