




<div class="loader">
  <p>loading</p>
  <div class="words">
    <span class="word">papers</span>
    <span class="word">stacks</span>
    <span class="word">science</span>
    <span class="word">citations</span>
    <span class="word">authors</span>
  </div>
</div>


<div class="lineloader"></div>

<style>
  .loader {
    color: rgb(124, 124, 124);
    font-family: "Poppins", sans-serif;
    font-weight: 500;
    font-size: 25px;
    -webkit-box-sizing: content-box;
    box-sizing: content-box;
    height: 40px;
    padding: 10px 10px;
    display: -webkit-box;
    display: -ms-flexbox;
    display: flex;
    border-radius: 8px;
  }

  .words {
    overflow: hidden;
  }

  .word {
    display: block;
    height: 100%;
    padding-left: 6px;
    color: #956afa;
    animation: spin_4991 4s infinite;
  }

  @keyframes spin_4991 {
    10% {
      -webkit-transform: translateY(-105%);
      transform: translateY(-105%);
    }

    25% {
      -webkit-transform: translateY(-100%);
      transform: translateY(-100%);
    }

    35% {
      -webkit-transform: translateY(-205%);
      transform: translateY(-205%);
    }

    50% {
      -webkit-transform: translateY(-200%);
      transform: translateY(-200%);
    }

    60% {
      -webkit-transform: translateY(-305%);
      transform: translateY(-305%);
    }

    75% {
      -webkit-transform: translateY(-300%);
      transform: translateY(-300%);
    }

    85% {
      -webkit-transform: translateY(-405%);
      transform: translateY(-405%);
    }

    100% {
      -webkit-transform: translateY(-400%);
      transform: translateY(-400%);
    }
  }

  .lineloader {
 --height-of-loader: 4px;
 --loader-color: var(--main-blue);
 width: 60%;
 height: var(--height-of-loader);
 border-radius: 30px;
 background-color: rgba(0,0,0,0.2);
 position: relative;
}

.lineloader::before {
 content: "";
 position: absolute;
 background: var(--loader-color);
 top: 0;
 left: 0;
 width: 0%;
 height: 100%;
 border-radius: 30px;
 animation: moving 1s ease-in-out infinite;
 ;
}

@keyframes moving {
 50% {
  width: 100%;
 }

 100% {
  width: 0;
  right: 0;
  left: unset;
 }
}


</style>
