use std::rc::Rc;

use models::{Topic, TOPIC_GRID_ROWS, TOPIC_GRID_COLS};
use gloo_net::http::Request;
use yew::{prelude::*, platform::spawn_local};

#[function_component(GeckoPage)]
pub fn page() -> Html {
  let topic = use_state(|| None);

  {
    let topic = topic.clone();
    use_effect_with_deps(move |()| {
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

        topic.set(Some(result));
      });
    }, ());
  }

  let topic = match topic.as_ref() {
    None => html! { <div>{"No server response"}</div> },
    Some(Ok(topic)) => html! {
      <TopicCard topic={topic}/>
    },
    // Some(Err(err)) => err,
    _ => html!{ <div/> },
  };

  html! {
    <div class="Gecko">
      { topic }
    </div>
  }
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