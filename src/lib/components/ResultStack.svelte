<script>
  export let queryResult;
  //buttons handler
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  function addPaper(paper) {
    dispatch("addPaper", {
      paper: paper,
    });
  }
//   queryResult = [
//     {
//       paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
//       url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
//       title: "Construction of the Literature Graph in Semantic Scholar",
//       abstract:
//         "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
//       year: 2018,
//       citationCount: 296,
//       openAccessPdf: {
//         url: "https://www.aclweb.org/anthology/N18-3011.pdf",
//         status: "HYBRID",
//       },
//       citationStyles: {
//         bibtex:
//           "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
//       },
//       authors: [
//         { authorId: "145585097", name: "Waleed Ammar" },
//         { authorId: "3458736", name: "Dirk Groeneveld" },
//         { authorId: "1857797", name: "Chandra Bhagavatula" },
//         { authorId: "46181066", name: "Iz Beltagy" },
//         { authorId: "46230609", name: "Miles Crawford" },
//         { authorId: "145612610", name: "Doug Downey" },
//         { authorId: "38092776", name: "Jason Dunkelberger" },
//         { authorId: "143718836", name: "Ahmed Elgohary" },
//         { authorId: "46411828", name: "Sergey Feldman" },
//         { authorId: "4480314", name: "Vu A. Ha" },
//         { authorId: "143967880", name: "Rodney Michael Kinney" },
//       ],
//     },
//     {
//       paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
//       url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
//       title: "Construction of the Literature Graph in Semantic Scholar",
//       abstract:
//         "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
//       year: 2018,
//       citationCount: 296,
//       openAccessPdf: {
//         url: "https://www.aclweb.org/anthology/N18-3011.pdf",
//         status: "HYBRID",
//       },
//       citationStyles: {
//         bibtex:
//           "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
//       },
//       authors: [
//         { authorId: "145585097", name: "Waleed Ammar" },
//         { authorId: "3458736", name: "Dirk Groeneveld" },
//         { authorId: "1857797", name: "Chandra Bhagavatula" },
//         { authorId: "46181066", name: "Iz Beltagy" },
//         { authorId: "46230609", name: "Miles Crawford" },
//         { authorId: "145612610", name: "Doug Downey" },
//         { authorId: "38092776", name: "Jason Dunkelberger" },
//         { authorId: "143718836", name: "Ahmed Elgohary" },
//         { authorId: "46411828", name: "Sergey Feldman" },
//         { authorId: "4480314", name: "Vu A. Ha" },
//         { authorId: "143967880", name: "Rodney Michael Kinney" },
//       ],
//     },
//     {
//       paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
//       url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
//       title: "Construction of the Literature Graph in Semantic Scholar",
//       abstract:
//         "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
//       year: 2018,
//       citationCount: 296,
//       openAccessPdf: {
//         url: "https://www.aclweb.org/anthology/N18-3011.pdf",
//         status: "HYBRID",
//       },
//       citationStyles: {
//         bibtex:
//           "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
//       },
//       authors: [
//         { authorId: "145585097", name: "Waleed Ammar" },
//         { authorId: "3458736", name: "Dirk Groeneveld" },
//         { authorId: "1857797", name: "Chandra Bhagavatula" },
//         { authorId: "46181066", name: "Iz Beltagy" },
//         { authorId: "46230609", name: "Miles Crawford" },
//         { authorId: "145612610", name: "Doug Downey" },
//         { authorId: "38092776", name: "Jason Dunkelberger" },
//         { authorId: "143718836", name: "Ahmed Elgohary" },
//         { authorId: "46411828", name: "Sergey Feldman" },
//         { authorId: "4480314", name: "Vu A. Ha" },
//         { authorId: "143967880", name: "Rodney Michael Kinney" },
//       ],
//     },
//     {
//       paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
//       url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
//       title: "Construction of the Literature Graph in Semantic Scholar",
//       abstract:
//         "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
//       year: 2018,
//       citationCount: 296,
//       openAccessPdf: {
//         url: "https://www.aclweb.org/anthology/N18-3011.pdf",
//         status: "HYBRID",
//       },
//       citationStyles: {
//         bibtex:
//           "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
//       },
//       authors: [
//         { authorId: "145585097", name: "Waleed Ammar" },
//         { authorId: "3458736", name: "Dirk Groeneveld" },
//         { authorId: "1857797", name: "Chandra Bhagavatula" },
//         { authorId: "46181066", name: "Iz Beltagy" },
//         { authorId: "46230609", name: "Miles Crawford" },
//         { authorId: "145612610", name: "Doug Downey" },
//         { authorId: "38092776", name: "Jason Dunkelberger" },
//         { authorId: "143718836", name: "Ahmed Elgohary" },
//         { authorId: "46411828", name: "Sergey Feldman" },
//         { authorId: "4480314", name: "Vu A. Ha" },
//         { authorId: "143967880", name: "Rodney Michael Kinney" },
//       ],
//     },
//     {
//       paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
//       url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
//       title: "Construction of the Literature Graph in Semantic Scholar",
//       abstract:
//         "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
//       year: 2018,
//       citationCount: 296,
//       openAccessPdf: {
//         url: "https://www.aclweb.org/anthology/N18-3011.pdf",
//         status: "HYBRID",
//       },
//       citationStyles: {
//         bibtex:
//           "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
//       },
//       authors: [
//         { authorId: "145585097", name: "Waleed Ammar" },
//         { authorId: "3458736", name: "Dirk Groeneveld" },
//         { authorId: "1857797", name: "Chandra Bhagavatula" },
//         { authorId: "46181066", name: "Iz Beltagy" },
//         { authorId: "46230609", name: "Miles Crawford" },
//         { authorId: "145612610", name: "Doug Downey" },
//         { authorId: "38092776", name: "Jason Dunkelberger" },
//         { authorId: "143718836", name: "Ahmed Elgohary" },
//         { authorId: "46411828", name: "Sergey Feldman" },
//         { authorId: "4480314", name: "Vu A. Ha" },
//         { authorId: "143967880", name: "Rodney Michael Kinney" },
//       ],
//     },
//     {
//       paperId: "649def34f8be52c8b66281af98ae884c09aef38b",
//       url: "https://www.semanticscholar.org/paper/649def34f8be52c8b66281af98ae884c09aef38b",
//       title: "Construction of the Literature Graph in Semantic Scholar",
//       abstract:
//         "We describe a deployed scalable system for organizing published scientific literature into a heterogeneous graph to facilitate algorithmic manipulation and discovery. The resulting literature graph consists of more than 280M nodes, representing papers, authors, entities and various interactions between them (e.g., authorships, citations, entity mentions). We reduce literature graph construction into familiar NLP tasks (e.g., entity extraction and linking), point out research challenges due to differences from standard formulations of these tasks, and report empirical results for each task. The methods described in this paper are used to enable semantic features in www.semanticscholar.org.",
//       year: 2018,
//       citationCount: 296,
//       openAccessPdf: {
//         url: "https://www.aclweb.org/anthology/N18-3011.pdf",
//         status: "HYBRID",
//       },
//       citationStyles: {
//         bibtex:
//           "@['JournalArticle', 'Conference']{Ammar2018ConstructionOT,\n author = {Waleed Ammar and Dirk Groeneveld and Chandra Bhagavatula and Iz Beltagy and Miles Crawford and Doug Downey and Jason Dunkelberger and Ahmed Elgohary and Sergey Feldman and Vu A. Ha and Rodney Michael Kinney and Sebastian Kohlmeier and Kyle Lo and Tyler C. Murray and Hsu-Han Ooi and Matthew E. Peters and Joanna L. Power and Sam Skjonsberg and Lucy Lu Wang and Christopher Wilhelm and Zheng Yuan and Madeleine van Zuylen and Oren Etzioni},\n booktitle = {North American Chapter of the Association for Computational Linguistics},\n pages = {84-91},\n title = {Construction of the Literature Graph in Semantic Scholar},\n year = {2018}\n}\n",
//       },
//       authors: [
//         { authorId: "145585097", name: "Waleed Ammar" },
//         { authorId: "3458736", name: "Dirk Groeneveld" },
//         { authorId: "1857797", name: "Chandra Bhagavatula" },
//         { authorId: "46181066", name: "Iz Beltagy" },
//         { authorId: "46230609", name: "Miles Crawford" },
//         { authorId: "145612610", name: "Doug Downey" },
//         { authorId: "38092776", name: "Jason Dunkelberger" },
//         { authorId: "143718836", name: "Ahmed Elgohary" },
//         { authorId: "46411828", name: "Sergey Feldman" },
//         { authorId: "4480314", name: "Vu A. Ha" },
//         { authorId: "143967880", name: "Rodney Michael Kinney" },
//       ],
//     },
//   ];
</script>

<div class="space">
  {#if queryResult}
    <div class="rack">
      {#each queryResult as paper, i}
        <div class="publication" on:click={addPaper(paper)}>
          <div class="header">
            <div class="smalltext">
              {paper.year} | {paper.citationCount} cit.
            </div>
            <h2>
              {paper.title}
            </h2>
          </div>

          <div class="smalltext">
            {paper.authors[0].name}
            <span style="font-style: italic"
              >{paper.authors.length > 1 ? " et all" : ""}</span
            >
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="description">
      Lorem ipsum dolor sit amet consectetur adipisicing elit. Vero suscipit
      nesciunt officia veritatis quia quis est ea. Officiis tempora adipisci
      necessitatibus nulla praesentium omnis nostrum quasi architecto, eveniet,
      sunt earum!
    </div>
  {/if}
</div>

<style>
  .space {
    width: 100%;
    height: 15rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
  }
  h2 {
    font-size: 0.8rem;
    font-family: "Lora", serif;
    font-weight: 800;
  }
  .rack {
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
  }

  .description {
    width: 50%;
    color: gray;
  }
  .publication {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    /* border: 1px solid rgba(0, 0, 0, 0.3); */
    border-radius: 5px;
    width: 9rem;
    aspect-ratio: 1/1.4;
    cursor: pointer;
    padding: 0.7rem;
    /* Functional props*/
    --rotation: calc(calc(30deg / -4) + calc(calc(30deg / 25)) * var(--order));
    transform: rotate(calc(var(--random) * 2deg))
      translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
    transform-origin: center;
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
    background-color: white;
    --order: 3;
    --shadow: calc(var(--order) * 1px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
      rgba(0, 0, 0, 0.25);
  }
  .publication:not(:first-child) {
    margin-left: -1.5rem;
  }
  .publication:hover {
    transform: translateY(-1rem);
  }

  .publication:hover ~ .publication {
    transform: translateX(1.5rem);
  }
  .smalltext {
    font-weight: 100;
    font-size: 0.75rem;
  }
</style>
