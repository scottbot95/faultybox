use std::rc::Rc;

use models::{Topic, TOPIC_GRID_ROWS, TOPIC_GRID_COLS};
use gloo_net::http::Request;
use yew::{prelude::*, platform::spawn_local};

#[function_component(GeckoPage)]
pub fn page() -> Html {
  let topic = use_state(|| None).clone();

  let topic_card = topic.as_ref().map(|res| {
    match res {
      Ok(topic) => html! {
        <TopicCard topic={topic}/>
      },
      Err(err) => html! {
        <div>
        {"An error occurred fetching topic from server"}
        {err}
        </div>
      },
    }
  });

  html! {
    <div class="Gecko">
      { topic_card }
      <button onclick={move |_| {
        fetch_topic(topic.setter());
      }} >{"New topic"}</button>
    </div>
  }
}

fn fetch_topic(setter: UseStateSetter<Option<Result<Rc<Topic>, String>>>) {
  spawn_local(async move {
    let resp = Request::get("/api/gecko/random-card").send().await;
    let result = match resp {
      Ok(resp) => if resp.ok() {
        resp.json::<Topic>()
            .await
            .map(Rc::new)
            .map_err(|err| err.to_string())
      } else {
        Err(format!(
          "Error fetching data {} ({})",
          resp.status(),
          resp.status_text(),
        ))
      }
      Err(err) => Err(err.to_string()),
    };

    setter.set(Some(result));
  });
}

#[derive(Properties, PartialEq)]
pub struct TopicProps {
  topic: Rc<Topic>,
}

#[function_component(TopicCard)]
pub fn topic_card(props: &TopicProps) -> Html {
  let topic = props.topic.as_ref();
  
  let word_rows = (0..TOPIC_GRID_ROWS).map(|row| {
    let cols = (0..TOPIC_GRID_COLS).map(|col| {
      let i = row * TOPIC_GRID_COLS + col;
      html!{
        <td key={col}>{&topic.words[i]}</td>
      }
    }).collect::<Html>();

    html! {
      <tr key={row}>
        <td>{row + 1}</td>
        {cols}
      </tr>
    }
  }).collect::<Html>();

  html! {
    <div class="Topic">
      <h1>{&topic.name}</h1>
      <table>
        <thead>
          <tr>
            // TODO this can probably be a loop?
            <th />
            <th>{'A'}</th>
            <th>{'B'}</th>
            <th>{'C'}</th>
            <th>{'D'}</th>
          </tr>
        </thead>
        <tbody>
          {word_rows}
        </tbody>
      </table>
    </div>
  }
}