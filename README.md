# sulfate-xml

an xml library, explicitly for SOAP messages

As a stepping stone towards building a SOAP client/server library, I want to be able to convert
structs to and from an intermediate XML representation so I have something standard to use with a
serializer/deserializer.

This library, at least in its early stages, is intentionally limited, as it's mainly intended as a
component of a (later) SOAP client/server library. At the moment, that means attributes other than
namespaces are not present. The XML library I currently have can support them, so if I find a need
for them (i figure i'll probably run into them eventually) it won't be much of an issue to add them
in.

<!-- vim: set tw=100 expandtab: -->
