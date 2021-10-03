module Main where

import Prelude

import Effect (Effect)
import Halogen as H
import Halogen.Aff as HA
import Halogen.HTML as HH
import Halogen.HTML.Properties as HP
import Halogen.VDom.Driver (runUI)
import Data.MediaType.Common as MTC
import Effect.Console (log)

main :: Effect Unit
main = HA.runHalogenAff do
  body <- HA.awaitBody
  runUI component unit body

type State = Unit

component :: forall query input output m. H.Component query input output m
component =
  H.mkComponent
    { initialState
    , render
    , eval: H.mkEval H.defaultEval
    }

initialState :: forall input. input -> State
initialState _ = unit

render :: forall m a. State -> H.ComponentHTML a () m
render _ =
  HH.form
    [ HP.action "/api/submit/", HP.method HP.POST, HP.enctype MTC.multipartFormData ]
    [ HH.label_ [ HH.text "ROM File to Upload: " ]
    , HH.input [ HP.type_ HP.InputFile, HP.name "rom_file" ]      
    , HH.div_
      [ HH.label_ [ HH.text "Seed:" ]
      , HH.input [ HP.type_ HP.InputText, HP.name "seed" ]
      ]
    , HH.div_
      [ HH.input [ HP.id "std", HP.type_ HP.InputRadio, HP.name "entrance_shuffle_type", HP.value "Standard" ]      
      , HH.label [ HP.for "std" ] [ HH.text "Standard Shuffle" ]
      , HH.br_
      , HH.input [ HP.id "chaos", HP.type_ HP.InputRadio, HP.name "entrance_shuffle_type", HP.value "Chaos" ]      
      , HH.label [ HP.for "chaos" ] [ HH.text "Chaos Shuffle" ]
      , HH.br_
      ]
    , HH.input [ HP.type_ HP.InputSubmit, HP.value "Submit" ]
    ]

