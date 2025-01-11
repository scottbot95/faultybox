use std::rc::Rc;

use gloo_net::http::Request;
use models::{Topic, TOPIC_GRID_COLS, TOPIC_GRID_ROWS};
use patternfly_yew::prelude::*;
use yew::{platform::spawn_local, prelude::*};

#[function_component(GeckoPage)]
pub fn page() -> Html {
    let topic = use_state(|| None);

    let topic_card = topic.as_ref().map(|res| match res {
        Ok(topic) => html! {
          <TopicCard topic={topic}/>
        },
        Err(err) => html! {
          <div>
            {"An error occurred fetching topic from server"}
            {err}
          </div>
        },
    });

    let new_topic = Callback::from(move |_| {
        let topic = topic.clone();
        spawn_local(async move {
            topic.set(Some(fetch_topic().await));
        });
    });
    html! {
      <div class="Gecko">
        <Grid>
          <GridItem cols={[2]} />
          <GridItem cols={[8]}>
            { topic_card }
          </GridItem>
          <GridItem cols={[2]} />
          <GridItem cols={[5]} />
          <GridItem cols={[2]}>
              <button onclick={new_topic}>{"New topic"}</button>
          </GridItem>
          <GridItem cols={[5]} />
        </Grid>
      </div>
    }
}

async fn fetch_topic() -> Result<Rc<Topic>, String> {
    let resp = Request::get("/api/gecko/random-card").send().await;
    match resp {
        Ok(resp) => {
            if resp.ok() {
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
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Properties, PartialEq)]
pub struct TopicProps {
    topic: Rc<Topic>,
}

#[function_component(TopicCard)]
pub fn topic_card(props: &TopicProps) -> Html {
    let topic = props.topic.as_ref();

    let word_rows = (0..TOPIC_GRID_ROWS)
        .map(|row| {
            let cols = (0..TOPIC_GRID_COLS)
                .map(|col| {
                    let i = row * TOPIC_GRID_COLS + col;
                    html! {
                      <td key={col}>{&topic.words[i]}</td>
                    }
                })
                .collect::<Html>();

            html! {
              <tr key={row}>
                <td>{row + 1}</td>
                {cols}
              </tr>
            }
        })
        .collect::<Html>();

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
