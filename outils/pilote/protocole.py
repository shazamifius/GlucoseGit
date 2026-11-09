"""Le protocole du pointeur virtuel de wlroots (`wlr-virtual-pointer-unstable-v1`), écrit à la main
d'après son XML (fiche 99) : le générateur de pywayland change de rangement d'une version à l'autre,
et ces deux interfaces tiennent en quelques lignes. Les requêtes se déclarent **dans l'ordre du
XML** : leur rang est leur code.
"""
from pywayland.protocol.wayland import WlSeat
from pywayland.protocol_core import Argument, ArgumentType, Global, Interface, Proxy, Resource

U, F, I = ArgumentType.Uint, ArgumentType.Fixed, ArgumentType.Int


class ZwlrVirtualPointerV1(Interface):
    name = "zwlr_virtual_pointer_v1"
    version = 2


class ZwlrVirtualPointerV1Proxy(Proxy[ZwlrVirtualPointerV1]):
    interface = ZwlrVirtualPointerV1

    @ZwlrVirtualPointerV1.request(Argument(U), Argument(F), Argument(F))
    def motion(self, time, dx, dy):
        self._marshal(0, time, dx, dy)

    @ZwlrVirtualPointerV1.request(Argument(U), Argument(U), Argument(U), Argument(U), Argument(U))
    def motion_absolute(self, time, x, y, x_extent, y_extent):
        self._marshal(1, time, x, y, x_extent, y_extent)

    @ZwlrVirtualPointerV1.request(Argument(U), Argument(U), Argument(U))
    def button(self, time, button, state):
        self._marshal(2, time, button, state)

    @ZwlrVirtualPointerV1.request(Argument(U), Argument(U), Argument(F))
    def axis(self, time, axis, value):
        self._marshal(3, time, axis, value)

    @ZwlrVirtualPointerV1.request()
    def frame(self):
        self._marshal(4)

    @ZwlrVirtualPointerV1.request(Argument(U))
    def axis_source(self, axis_source):
        self._marshal(5, axis_source)

    @ZwlrVirtualPointerV1.request(Argument(U), Argument(U))
    def axis_stop(self, time, axis):
        self._marshal(6, time, axis)

    @ZwlrVirtualPointerV1.request(Argument(U), Argument(U), Argument(F), Argument(I))
    def axis_discrete(self, time, axis, value, discrete):
        self._marshal(7, time, axis, value, discrete)

    @ZwlrVirtualPointerV1.request()
    def destroy(self):
        self._marshal(8)
        self._destroy()


class ZwlrVirtualPointerV1Resource(Resource):
    interface = ZwlrVirtualPointerV1


class ZwlrVirtualPointerV1Global(Global):
    interface = ZwlrVirtualPointerV1


ZwlrVirtualPointerV1._gen_c()
ZwlrVirtualPointerV1.proxy_class = ZwlrVirtualPointerV1Proxy
ZwlrVirtualPointerV1.resource_class = ZwlrVirtualPointerV1Resource
ZwlrVirtualPointerV1.global_class = ZwlrVirtualPointerV1Global


class ZwlrVirtualPointerManagerV1(Interface):
    name = "zwlr_virtual_pointer_manager_v1"
    version = 2


class ZwlrVirtualPointerManagerV1Proxy(Proxy[ZwlrVirtualPointerManagerV1]):
    interface = ZwlrVirtualPointerManagerV1

    @ZwlrVirtualPointerManagerV1.request(
        Argument(ArgumentType.Object, interface=WlSeat, nullable=True),
        Argument(ArgumentType.NewId, interface=ZwlrVirtualPointerV1),
    )
    def create_virtual_pointer(self, seat):
        return self._marshal_constructor(0, ZwlrVirtualPointerV1, seat)

    @ZwlrVirtualPointerManagerV1.request()
    def destroy(self):
        self._marshal(1)
        self._destroy()


class ZwlrVirtualPointerManagerV1Resource(Resource):
    interface = ZwlrVirtualPointerManagerV1


class ZwlrVirtualPointerManagerV1Global(Global):
    interface = ZwlrVirtualPointerManagerV1


ZwlrVirtualPointerManagerV1._gen_c()
ZwlrVirtualPointerManagerV1.proxy_class = ZwlrVirtualPointerManagerV1Proxy
ZwlrVirtualPointerManagerV1.resource_class = ZwlrVirtualPointerManagerV1Resource
ZwlrVirtualPointerManagerV1.global_class = ZwlrVirtualPointerManagerV1Global
