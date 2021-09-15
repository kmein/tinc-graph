def average: add / length;

def statProperty(property):
  to_entries
  | map(select(.value | property))
  | max_by(.value | property) as $max
  | min_by(.value | property) as $min
  | {
    max: $max | .value | property,
    maxNode: $max | .key,
    min: $min | .value | property,
    minNode: $min | .key,
    avg: map(.value | property) | average
  };

{
  longitude: statProperty(.longitude),
  latitude: statProperty(.latitude),
  connections: statProperty(.to | length),
  weight:
    to_entries
    | map(
      .key as $source
      | .value.to
      | map({key: ($source + "→" + .name), value: .})
    )
    | flatten
    | from_entries
    | statProperty(.weight),
}
